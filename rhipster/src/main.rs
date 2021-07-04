use anyhow::Error;
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use scan_dir::ScanDir;
use sourcegen_cli::{run_sourcegen, SourceGenerator, SourcegenParameters};
use syn::{AttributeArgs, ItemMod, Meta, NestedMeta};

use diesel_cli::config::Config;
use diesel_cli::print_schema::run_print_schema;
use diesel_cli::{run_migrations_with_output, FileBasedMigrations};
#[macro_use]
extern crate diesel_cli;

use diesel_cli_ext::parse::parse;
use diesel_cli_ext::print_normal_dependencies;

use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::Path;

const TEMPLATE: Dir = include_dir!("templates/main");
const MIGRATIONS: Dir = include_dir!("templates/migrations");

fn main() {
    // Collect all input
    let mut args: Vec<String> = std::env::args().collect();
    args.reverse();
    args.pop().unwrap();
    let target_name = args.pop().unwrap();
    let database_url = args.pop().unwrap();
    let up_file = args.pop().unwrap();
    let down_file = args.pop().unwrap();

    // Unpack templated project
    std::fs::create_dir(&target_name).unwrap();
    TEMPLATE.extract(&target_name).unwrap();
    MIGRATIONS
        .extract(format!("{}/migrations", &target_name))
        .unwrap();

    // Run a global replace on the config files
    ScanDir::files()
        .read(&target_name, |iter| {
            for (entry, _) in iter {
                let path = entry.path();
                let content = read_to_string(&path).unwrap();
                let content = content.replace("##DATABASE##", &database_url);
                let content = content.replace("##NAME##", &target_name);

                let mut f = File::create(&path).unwrap();
                f.write_all(content.as_bytes()).unwrap();
            }
        })
        .unwrap();

    // Copy the migration files into place
    let content = read_to_string(&up_file).unwrap();
    let mut f = File::create(format!(
        "{}/migrations/00000000000001_init_tables/up.sql",
        &target_name
    ))
    .unwrap();
    f.write_all(content.as_bytes()).unwrap();
    let content = read_to_string(&down_file).unwrap();
    let mut f = File::create(format!(
        "{}/migrations/00000000000001_init_tables/down.sql",
        &target_name
    ))
    .unwrap();
    f.write_all(content.as_bytes()).unwrap();

    // Run database migrations
    run_migrations(&database_url, &format!("{}/migrations", &target_name));

    // Create the Diesel ORM schema
    let schema = create_schema(&database_url, &target_name);
    let mut f = File::create(format!("{}/src/schema.rs", &target_name)).unwrap();
    f.write_all(schema.as_bytes()).unwrap();

    // Create Rust structs for the models
    let (models, list) = create_models(&schema);
    let list: Vec<_> = list
        .into_iter()
        .map(|v| v.strip_suffix("s").unwrap().to_string())
        .collect();

    // Drop models and sourcegen targets in place
    let mut model_string = models;
    for item in list.iter() {
        model_string += &format!(
            r#"
use schema::{0}s;
#[sourcegen::sourcegen(generator = "crud_generator", model = "{0}")]
mod {0}s {{}}"#,
            item
        );
    }

    // Wire up the Rocket routes
    let mut routes = String::new();
    for item in list {
        routes += &format!(
            r#"
                {0}s_mod::get_{0}s,
                {0}s_mod::get_{0},
                {0}s_mod::post_{0},
                {0}s_mod::delete_{0},
                "#,
            item
        );
    }

    // Run a global replace on the source files
    ScanDir::files()
        .read(format!("{}/src", &target_name), |iter| {
            for (entry, _) in iter {
                let path = entry.path();
                let content = read_to_string(&path).unwrap();
                let content = content.replace("##DATABASE##", &database_url);
                let content = content.replace("##NAME##", &target_name);
                let content = content.replace("##ROUTES##", &routes);
                let content = content.replace("##MODELS##", &model_string);

                let mut f = File::create(&path).unwrap();
                f.write_all(content.as_bytes()).unwrap();
            }
        })
        .unwrap();

    // Run sourcegen to generate Rocket code
    let path = format!("{}/Cargo.toml", &target_name);
    let parameters = SourcegenParameters {
        manifest: Some(Path::new(&path)),
        generators: &[("crud_generator", &CrudGenerator {})],
        ..Default::default()
    };
    run_sourcegen(&parameters).unwrap();
}

/// Run model migrations against the database.
fn run_migrations(database_url: &str, dir: &str) {
    let dir = FileBasedMigrations::from_path(dir).unwrap();
    call_with_conn!(database_url, run_migrations_with_output(dir)).unwrap();
}

/// Generate the Diesel ORM schema from the database using `diesel_cli`
fn create_schema(database_url: &str, target_name: &str) -> String {
    let path = format!("{}/diesel.toml", target_name);
    let path = Path::new(&path);
    let config = (|| -> Result<_, Error> {
        if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)?.read_to_end(&mut bytes)?;
            let mut result = toml::from_slice::<Config>(&bytes)?;
            result.set_relative_path_base(path.parent().unwrap());
            Ok(result)
        } else {
            Ok(Config::default())
        }
    })()
    .unwrap()
    .print_schema;

    let mut schema = Vec::new();
    run_print_schema(&database_url, &config, &mut schema).unwrap();

    std::str::from_utf8(schema.as_slice()).unwrap().to_string()
}

/// Create models from the Diesel schema using `diesel_cli_ext`
fn create_models(schema: &str) -> (String, Vec<String>) {
    let contents = schema.to_string();

    let mut output = String::new();

    let parse_output = parse(
        contents.clone(),
        "model",
        Some("Queryable, Serialize".to_string()),
        false,
        &mut HashMap::new(),
        false,
    );
    output = print_normal_dependencies(&parse_output, output);
    output += "\n";
    output += &parse_output.str_model;

    output += "\n";

    let parse_output = parse(
        contents,
        "model",
        Some("Insertable, Deserialize".to_string()),
        true,
        &mut HashMap::new(),
        true,
    );
    output += &parse_output.str_model;

    (output, parse_output.model_list)
}

/// Use `sourcegen` to generate code for CRUD Rocket routes
struct CrudGenerator {}

impl SourceGenerator for CrudGenerator {
    fn generate_mod(
        &self,
        args: AttributeArgs,
        _item: &ItemMod,
    ) -> Result<Option<TokenStream>, Error> {
        let core = args
            .iter()
            .find_map(|v| {
                if let NestedMeta::Meta(Meta::NameValue(nv)) = v {
                    if let syn::Lit::Str(ref value) = nv.lit {
                        if nv.path.is_ident("model") {
                            return Some(value.value());
                        }
                    }
                }

                None
            })
            .unwrap();

        let core = core.to_case(Case::Camel);

        let pluralized = format!("{}s", core.to_case(Case::Camel));
        let pluralized_ident = Ident::new(&pluralized, Span::call_site());
        let capitalized_ident = Ident::new(&core.to_case(Case::Pascal), Span::call_site());
        let new_ident = Ident::new(
            &format!("New{}", core.to_case(Case::Pascal)),
            Span::call_site(),
        );
        let mod_ident = Ident::new(&format!("{}_mod", &pluralized), Span::call_site());

        let raw_path = format!("/{}", pluralized);
        let id_path = format!("/{}/<id>", pluralized);

        let list_ident = Ident::new(&format!("get_{}", pluralized), Span::call_site());
        let get_ident = Ident::new(&format!("get_{}", core), Span::call_site());
        let post_ident = Ident::new(&format!("post_{}", core), Span::call_site());
        let delete_ident = Ident::new(&format!("delete_{}", core), Span::call_site());

        let core_ident = Ident::new(&core, Span::call_site());

        let output = quote! {
            mod #mod_ident{
                use super::*;

                #[get(#raw_path)]
                pub(crate) async fn #list_ident(db: Database) -> Json<Vec<#capitalized_ident>> {
                    let #pluralized_ident = db
                        .run(|c| #pluralized_ident::table.load::<#capitalized_ident>(c).unwrap())
                        .await;

                    Json(#pluralized_ident)
                }

                #[get(#id_path)]
                pub(crate) async fn #get_ident(db: Database, id: i32) -> Result<Json<#capitalized_ident>, ()> {
                    let #core_ident = db
                        .run(move |c| #pluralized_ident::table.find(id).first::<#capitalized_ident>(c))
                        .await
                        .map_err(|_| ())?;

                    Ok(Json(#core_ident))
                }

                #[post(#raw_path, data = "<data>")]
                pub(crate) async fn #post_ident(
                    db: Database,
                    data: Json<#new_ident>,
                ) -> Result<(), ()> {
                    db.run(|c| {
                        diesel::insert_into(#pluralized_ident::table)
                            .values(data.into_inner())
                            .execute(c)
                    })
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }

                #[delete(#id_path)]
                pub(crate) async fn #delete_ident(db: Database, id: i32) -> Result<(), ()> {
                    db.run(move |c| {
                        diesel::delete(#pluralized_ident::table.filter(#pluralized_ident::id.eq(id))).execute(c)
                    })
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }
            }
        };

        Ok(Some(TokenStream::from(output)))
    }
}
