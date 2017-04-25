#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(i128_type)]

#[macro_use]
extern crate nom;

mod structs;

use structs::*;
use std::io;
use std::fs;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::mem;
use nom::IResult::Done;

const FACTORIO_BASE: &'static str = "C:\\Program Files\\Factorio\\";
const BASE_RECIPE_PATH: &'static str = "data\\base\\prototypes\\recipe\\";

#[derive(Debug, Clone)]
struct LuaObject {
    fields: Vec<(Option<String>, LuaField)>
}

#[derive(Debug, Clone)]
enum LuaField {
    AString(String),
    AFloat(f64),
    ABool(bool),
    AnObject(Box<LuaObject>)
}

use LuaField::*;

named!(parse_bool<bool>, alt!(
    tag!("false") => {|_| false} |
    tag!("true")  => {|_| true} 
));

named!(parse_string<&str>, alt!(
    chain!(
        tag!("\"")        ~
        s: is_not!("\"")  ~
        tag!("\"")        ,
        || {str::from_utf8(s).unwrap()}
    ) => {|x| x} |
    tag!("\"\"") => {|_| ""}
));

named!(parse_float<f64>, chain!(
    minus: tag!("-")?            ~
    integer: is_a!("0123456789") ~
    fraction: chain!(
        tag!(".")              ~
        s: is_a!("0123456789") ,
        || {s}
    )?                           ,
    || {
        let sign = if minus.is_some() {-1.0} else {1.0};
        let mut s = str::from_utf8(integer).unwrap().to_string();
        if let Some(frac) = fraction {
            s = format!("{}.{}", s, str::from_utf8(frac).unwrap());
        }
        sign * s.parse::<f64>().unwrap()
    }
));

named!(spaces, is_a!(" \t\r\n"));

named!(ident, is_a!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"));

named!(parse_field<(Option<String>, LuaField)>, chain!(
    name: chain!(
        name: call!(ident)             ~
        call!(spaces)?                 ~
        tag!("=")                      ~
        call!(spaces)?                 ,
        || {str::from_utf8(name).unwrap().to_string()}
    )?                        ~
    field: call!(parse_value) ,
    || {(name, field)}
));

named!(parse_value<LuaField>, alt!(
    call!(parse_bool)   => {|x: bool| ABool(x)} |
    call!(parse_string) => {|x: &str| AString(x.to_string())} |
    call!(parse_float)  => {|x: f64|  AFloat(x)} |
    call!(parse_object) => {|x|       AnObject(box x) }
));

named!(parse_object<LuaObject>, chain!(
    tag!("{")                 ~
    call!(spaces)?            ~
    all_but_one: many0!(chain!(
        field: call!(parse_field) ~
        call!(spaces)?            ~
        tag!(",")                 ~
        call!(spaces)?            ,
        || {field}
    ))                        ~
    last: call!(parse_field)? ~
    call!(spaces)?            ~
    tag!("}")                 ,
    || {
        if last.is_none() && all_but_one.len() == 0 {
            return LuaObject{fields: Vec::new()};
        }
        let mut vec = all_but_one;
        if last.is_some() {
            vec.push(last.unwrap());
        }
        LuaObject{fields: vec}
    }
));

named!(parse_file< Vec<LuaObject> >, chain!(
    call!(spaces)?           ~
    tag!("data:extend(")     ~
    call!(spaces)?           ~
    obj: call!(parse_object) ~
    call!(spaces)?           ~
    tag!(")")                ,
    || {
        obj.fields.iter()
            .map(|x| match x { 
                &(_, AnObject(box ref obj)) => obj.clone(), 
                _ => panic!("Top level primitive found")
            })
            .collect()
    }
));

fn run() -> Result<(), io::Error> {
    let out_path = Path::new("new.data");
    let mut out_file = File::create(&out_path)?;
    let dir_path = format!("{}{}", FACTORIO_BASE, BASE_RECIPE_PATH);
    let dirs = fs::read_dir(dir_path)?;
    let mut sum = 0;
    let mut set = HashSet::new();
    for dir in dirs {
        let path_buf = dir?.path();
        let path = path_buf.as_path();
        let mut file = File::open(&path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let file = parse_file(s.as_bytes());
        if let Done(_, mut vec) = file {
            sum += vec.len();
            for elem in vec {
                let recipe = to_recipe(elem);
                set.insert(recipe.category.clone());
                print_recipe(&mut out_file, &recipe);
            }
        } else {
        }
    }
    println!("Total: {}", sum);
    println!("Categories: ");
    for s in set {
        println!("\t{}", s);
    }
    Ok(())
}

fn to_recipe(obj: LuaObject) -> Recipe {
    let obj_str = format!("{:?}", obj);
    let mut o_name = None;
    let mut o_category = "crafting".to_string();
    let mut o_products = None;
    let mut o_ingredients = None;
    let mut o_energy_required = 0.5;
    let mut o_enabled = false;
    for (opt_name, field) in obj.fields {
        if let Some(name) = opt_name {
            match name.as_ref() {
                "type" => {
                    if let AString(s) = field {
                        assert_eq!(s, "recipe");
                    } else {
                        panic!("\"type\" is wrong: {:?}", field);
                    }
                },
                "name" => {
                    if let AString(s) = field {
                        o_name = Some(s);
                    } else {
                        panic!("\"name\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "category" => {
                    if let AString(s) = field {
                        o_category = s;
                    } else {
                        panic!("\"category\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "enabled" => {
                    if let ABool(x) = field {
                        o_enabled = x;
                    } else if let AString(ref s) = field {
                        o_enabled = match s.as_ref() {
                            "true" => true,
                            "false" => false,
                            &_ => panic!("Try to convert string to bool, but failed: {:?} ({})", field, obj_str)
                        };
                    } else {
                        panic!("\"enabled\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                }
                "energy_required" => {
                    if let AFloat(x) = field {
                        o_energy_required = x;
                    } else {
                        panic!("\"energy_required\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "result" => {
                    if let AString(s) = field {
                        o_products = Some(vec![RecipeComponent {
                            a_type: "item".to_string(),
                            name: s,
                            amount: 1.0
                        }]);
                    } else {
                        panic!("\"result\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "result_count" => {
                    if let AFloat(x) = field {
                        if let &mut Some(ref mut vec) = &mut o_products {
                            vec[0].amount = x;
                        }
                    } else {
                        panic!("\"result_count\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "results" => {
                    if let AnObject(box obj) = field {
                        o_products = Some(to_components(obj));
                    } else {
                        panic!("\"results\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                "ingredients" => {
                    if let AnObject(box obj) = field {
                        o_ingredients = Some(to_components(obj));
                    } else {
                        panic!("\"ingredients\" of inappropriate type: {:?} ({})", field, obj_str);
                    }
                },
                &_ => ()
            };
        }
    }
    if o_name.is_none() {
        panic!("Recipe with no name! ({})", obj_str);
    }
    if o_products.is_none() {
        panic!("Recipe with no products! ({})", obj_str);
    }
    if o_ingredients.is_none() {
        panic!("Recipe with no ingredients! ({})", obj_str);
    }
    Recipe {
        name: o_name.unwrap(),
        category: o_category,
        products: o_products.unwrap(),
        ingredients: o_ingredients.unwrap(),
        energy_required: o_energy_required,
        enabled: o_enabled
    }
}

fn to_components(obj: LuaObject) -> Vec<RecipeComponent> {
    let mut vec = Vec::new();
    for obj in obj.fields {
        if let (_, AnObject(mut obj)) = obj {
            let mut some = false;
            let mut none = false;
            for i in 0..obj.fields.len() {
                match obj.fields[i] {
                    (Some(_), _) => some = true,
                    (None, _) => none = true
                }
            }
            if some && none {
                panic!("Both named and unnamed fields: {:?}", obj);
            }
            if some {
                let mut m: HashMap<_, _> = HashMap::new();
                for (name, field) in obj.fields {
                    m.insert(name.unwrap(), field);
                }
                vec.push(RecipeComponent {
                    a_type: from_string(m.remove("type").unwrap()),
                    name: from_string(m.remove("name").unwrap()),
                    amount: from_float(m.remove("amount").unwrap())
                });
            } else {
                vec.push(RecipeComponent {
                    a_type: "item".to_string(),
                    name: from_string(mem::replace(&mut obj.fields[0], (None, ABool(false))).1),
                    amount: from_float(mem::replace(&mut obj.fields[1], (None, ABool(false))).1)
                });
            }
        } else {
            panic!("Primitives on top level of components: {:?}", obj);
        }
    }
    vec
}

fn print_recipe(file: &mut File, recipe: &Recipe) {
    if recipe.products.len() > 1 {
        return;
    }
    let ref product = recipe.products[0];
    write!(file, "{}\t{}\t{}\t\"", product.name, product.amount, recipe.energy_required);
    for i in 0..recipe.ingredients.len() {
        if i > 0 {
            write!(file, " | ");
        }
        let ref comp = recipe.ingredients[i];
        write!(file, "{} x {}", comp.amount, comp.name);
    }
    writeln!(file, "\"");
}

fn from_string(field: LuaField) -> String {
    if let AString(s) = field {
        s
    } else {
        panic!("Not a string: {:?}", field);
    }
}

fn from_float(field: LuaField) -> f64 {
    if let AFloat(x) = field {
        x
    } else {
        panic!("Not a float: {:?}", field);
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{:?}", err);
    }
}
