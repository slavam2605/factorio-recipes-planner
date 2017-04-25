use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;

#[derive(Debug)]
struct Recipe {
    output: f32,
    time: f32,
    compounds: Vec<(f32, String)>,
    time_factor: f32
}

#[derive(Debug)]
struct Edge {
    from: String,
    to: String,
    weight: f32
}

#[derive(Debug)]
struct Graph {
    vertices: HashSet<String>,
    edges: Vec<Edge>
}

#[derive(Debug)]
struct Param {
    time: Option<f32>,
    rate: f32
}

#[derive(Debug)]
struct AssemblePlan {
    out: Vec<(String, Param)>,
    compounds: HashMap<String, Param>
}

impl Graph {
    fn new() -> Graph {
        Graph { vertices: HashSet::new(), edges: Vec::new() }
    }
}

fn build_dependency_net(dependencies: &HashMap<&str, Recipe>, target: String) -> Graph {
    let mut graph = Graph::new();
    let mut queue = VecDeque::new();
    queue.push_back(target);
    while !queue.is_empty() {
        let v = queue.pop_front().unwrap();
        if graph.vertices.contains(&v) {
            continue;
        }
        graph.vertices.insert(v.clone());
        let recipe = match dependencies.get(&*v) {
            Some(a) => a,
            None => continue
        };
        for &(w, ref to) in recipe.compounds.iter() {
            graph.edges.push(Edge { from: v.clone(), to: to.clone(), weight: w });
            queue.push_back(to.clone());
        }
    }
    graph
}

fn make_plan(dependencies: &HashMap<&str, Recipe>, target: String, target_rate: f32, graph: Graph) -> AssemblePlan {
    let mut plan: HashMap<String, Param> = HashMap::new();
    let mut out: Vec<(String, Param)> = Vec::new();
    let mut undone = HashSet::new();
    for s in graph.vertices.iter() {
        undone.insert(s.clone());
    }

    while !undone.is_empty() {
        let mut v = None;
        for s in &undone {
            if !undone.contains(s) {
                break;
            }
            let mut flag = true;
            for edge in &graph.edges {
                if edge.to == *s && undone.contains(&edge.from) {
                    flag = false;
                    break;
                }
            }
            if flag {
                v = Some(s.clone());
                break;
            }
        }
        match v {
            None => panic!("No leaf found with nonempty unevaluated state!"),
            Some(v) => {
                undone.remove(&v);
                let time = dependencies.get(&*v).map(|r| r.time / r.time_factor);
                let mut rate = 0.0;
                for edge in &graph.edges {
                    if edge.to == v {
                        rate += edge.weight * plan.get(&edge.from).unwrap().rate;
                    }
                }
                if v == target {
                    rate = target_rate;
                }
                plan.insert(v.clone(), Param{time: time, rate: rate});
                out.push((v, Param{time: time, rate: rate}));
            }
        };
    }
    AssemblePlan{out: out, compounds: plan}
}

fn main() {
    let path = Path::new("new.data");
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(why) => panic!("error while open file: {}", why.description())
    };
    let mut s = String::new();
    if let Err(why) = file.read_to_string(&mut s) {
        panic!("error while read from file: {}", why.description())
    };
    let mut m: HashMap<_, _> = HashMap::new();
    for line in s.split("\n") {
        if line.starts_with("#") {
            continue;
        }
        let parts: Vec<_> = line.split("\t").collect();
        if parts.len() < 4 {
            println!("Not enough parts: {} of 4", parts.len());
            continue;
        }
        let compounds: Vec<_> = parts[3].replace("\"", "").split("|").map(|s| {
            let v: Vec<_> = s.split(" x ").collect();
            (match f32::from_str(v[0].trim()) {
                Ok(a) => a,
                Err(why) => panic!("error while parsing compounds ({})[{}]: {}", s, v[0], why.description())
            }, v[1].trim().to_string())
        }).collect();
        let time_factor = match compounds.len() {
            1...2 => 0.75, // FIXME 0.5
            3...4 => 0.75,
            5...6 => 1.25,
            _ => panic!("illegal number of components: {}", compounds.len())
        };
        let mut recipe = Recipe {
            output: match f32::from_str(parts[1]) {
                Ok(a) => a,
                Err(why) => {
                    println!("Not parsed 'output' of {}[{}]: {}", parts[0], parts[1], why.description());
                    continue;
                }
            },
            time: match f32::from_str(parts[2]) {
                Ok(a) => a,
                Err(why) => {
                    println!("Not parsed 'time' of {}[{}]: {}", parts[0], parts[2], why.description());
                    continue;
                }
            },
            compounds: compounds,
            time_factor: time_factor
        };
        let output = recipe.output;
        if output != 1.0 {
            recipe.output /= output;
            recipe.time /= output;
            for &mut (ref mut w, _) in &mut recipe.compounds {
                *w /= output;
            }
        }
        m.insert(parts[0], recipe);
    }

    let name = "electronic-circuit".to_string();
    println!("{:?}", m);
    let graph = build_dependency_net(&m, name.clone());
    println!("Components:");
    for component in &graph.vertices {
        println!("    {}", component);
    }
    println!("Edges:");
    for &Edge { ref from, ref to, weight } in graph.edges.iter() {
        println!("    {} --{}--> {}", from, weight, to);
    }
    let plan = make_plan(&m, name.clone(), 1.5 / 40.0, graph);
    println!("Assemble plan:");
    for &(ref component, Param{time, rate}) in &plan.out{
        if let Some(time) = time {
            println!("    {}: (time = {}) * (rate = {} ({} parts/min)) = (count = {})", component, time, rate, rate * 60.0, time * rate);
        }
    }
    println!("Components flow rate:");
    for &(ref component, Param{time, rate}) in &plan.out{
        if let None = time {
            println!("    {}: rate = {} parts/min", component, rate * 60.0);
        }
    }
}