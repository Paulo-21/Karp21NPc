use std::fs;

#[derive(Debug)]
struct Instance {
    m: usize,              // nombre de lignes
    n: usize,              // nombre de colonnes
    costs: Vec<i32>,       // coût de chaque colonne
    rows: Vec<Vec<usize>>, // colonnes qui couvrent chaque ligne
}

fn parse_instance() -> Instance {
    let file = fs::read_to_string("../instances/scp41.txt").unwrap();

    // Tous les nombres du fichier, indépendamment des retours à la ligne.
    let mut tokens = file.split_ascii_whitespace();

    // Première ligne : m n
    let m: usize = tokens.next().unwrap().parse().unwrap();
    let n: usize = tokens.next().unwrap().parse().unwrap();

    println!("m = {m}, n = {n}");

    // Les n coûts des colonnes
    let mut costs = Vec::with_capacity(n);

    for _ in 0..n {
        let cost: i32 = tokens.next().unwrap().parse().unwrap();
        costs.push(cost);
    }

    // Les m lignes de couverture
    let mut rows = Vec::with_capacity(m);

    for i in 0..m {
        // Nombre de colonnes couvrant cette ligne
        let k: usize = tokens
            .next()
            .unwrap_or_else(|| panic!("Missing k for row {i}"))
            .parse()
            .unwrap();

        let mut covered_by = Vec::with_capacity(k);

        for _ in 0..k {
            let col: usize = tokens.next().unwrap().parse().unwrap();
            // Rust utilise 0..n-1.
            covered_by.push(col - 1);
        }

        rows.push(covered_by);
    }

    Instance { m, n, costs, rows }
}
fn init_ti(instance: &Instance) -> Vec<f32> {
    let mut ti = Vec::with_capacity(instance.n);
    for i in 0..instance.n {}
    ti
}
fn solve(mut instance: Instance) -> f32 {
    let zmax = i32::MIN;
    let zUB = i32::MIN;
    let zLB = 0;
    let mut pk: Vec<i32> = vec![0; instance.n];
    pk.copy_from_slice(&instance.costs[0..instance.n]);
    0.6
}
fn main() {
    let instance = parse_instance();

    println!("m = {}", instance.m);
    println!("n = {}", instance.n);
    solve(instance);
}
