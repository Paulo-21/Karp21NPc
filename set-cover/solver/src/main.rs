use std::{env, f64, fs, time::Instant};

#[derive(Debug)]
struct Instance {
    m: usize,              // nombre de lignes
    n: usize,              // nombre de colonnes
    costs: Vec<i32>,       // coût de chaque colonne
    rows: Vec<Vec<usize>>, // colonnes qui couvrent chaque ligne
    covering_map: Vec<Vec<usize>>,
}

fn parse_instance(path: &str) -> Instance {
    let file = fs::read_to_string(path).unwrap();

    // Tous les nombres du fichier, indépendamment des retours à la ligne.
    let mut tokens = file.split_ascii_whitespace();

    // Première ligne : m n
    let m: usize = tokens.next().unwrap().parse().unwrap();
    let n: usize = tokens.next().unwrap().parse().unwrap();

    println!("m = {m}, n = {n}");

    // Les n coûts des colonnes
    let mut costs = Vec::with_capacity(n);
    let mut covering_score: Vec<u32> = vec![0; n];
    let mut idx: Vec<usize> = Vec::with_capacity(n);
    for id in 0..n {
        let cost: i32 = tokens.next().unwrap().parse().unwrap();
        costs.push(cost);
        idx.push(id);
    }
    idx.sort_by_key(|&k| costs[k]);
    costs.sort();
    let mut inverse = vec![0usize; n];
    for (en, &id) in idx.iter().enumerate() {
        inverse[id] = en;
    }
    // Les m lignes de couverture
    let mut rows = Vec::with_capacity(m);
    let mut covering_map = vec![Vec::new(); n];

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
            covered_by.push(inverse[col - 1]);
            covering_score[inverse[col - 1]] += 1;
            covering_map[inverse[col - 1]].push(i);
        }
        rows.push(covered_by);
    }
    for covered_by in rows.iter_mut() {
        covered_by.sort_by(|&a, &b| {
            costs[a]
                .cmp(&costs[b])
                .then_with(|| covering_score[b].cmp(&covering_score[a]))
        });
    }

    Instance {
        m,
        n,
        costs,
        rows,
        covering_map,
    }
}
fn init_ti(instance: &Instance) -> Vec<f64> {
    let mut ti = Vec::with_capacity(instance.m);
    for i in 0..instance.m {
        let mut min = i32::MAX;
        // rows[i] contient les colonnes qui couvrent la ligne i
        for &j in &instance.rows[i] {
            min = min.min(instance.costs[j]);
        }
        ti.push(min as f64);
    }
    ti
}
fn solve(instance: Instance) -> f64 {
    let debug = false;
    let start = Instant::now();
    let mut z_max = f64::NEG_INFINITY;
    let mut z_ub = f64::MAX;
    let mut z_lb = 0.;
    let mut pk: Vec<f64> = instance.costs.iter().map(|f| *f as f64).collect();
    let mut ti = init_ti(&instance);
    let mut x = vec![false; instance.n];
    let mut c_big = vec![0.0; instance.n];
    let mut g_big: Vec<f64> = vec![0.0; instance.m];
    let mut f = 2.0;
    let mut changed_zub = 0;
    let mut iteration = 0;
    let mut sol = Vec::with_capacity(instance.n);
    let mut last_zmax = z_max;
    let mut eliminated = vec![false; instance.n];
    let mut last_x = vec![false; instance.n];
    let mut covering_number = vec![0; instance.m];
    let mut sol_mark = vec![0usize; instance.n];
    let mut sol_generation = 0usize;

    println!("Init : {} ms", start.elapsed().as_millis());
    loop {
        sol_generation += 1;
        let start = Instant::now();
        // etape 2

        for j in 0..instance.n {
            if eliminated[j] {
                c_big[j] = f64::MAX; // Ignorer cette colonne
                x[j] = false;
                continue;
            }
            c_big[j] = instance.costs[j] as f64;
        }

        for i in 0..instance.m {
            for &j in &instance.rows[i] {
                if !eliminated[j] {
                    c_big[j] -= ti[i];
                }
            }
        }

        for j in 0..instance.n {
            if !eliminated[j] {
                x[j] = c_big[j] <= 0.0;
            }
        }
        z_lb = 0.;
        for j in 0..instance.n {
            if !eliminated[j] && x[j] {
                z_lb += c_big[j];
            }
        }
        for i in 0..instance.m {
            z_lb += ti[i];
        }
        // Update 2
        let start2 = Instant::now();
        z_max = z_max.max(z_lb);
        // Etape 3
        for (idx, x1) in x.iter().enumerate() {
            if *x1 != last_x[idx] {
                if *x1 {
                    for &cov in instance.covering_map[idx].iter() {
                        covering_number[cov] += 1;
                    }
                } else {
                    for &cov in instance.covering_map[idx].iter() {
                        covering_number[cov] -= 1;
                    }
                }
            }
        }
        sol.clear();

        // 3(a) : Ajouter les colonnes où X_j = 1
        for (j, &xp) in x.iter().enumerate() {
            if xp {
                sol.push((j, true));
                sol_mark[j] = sol_generation;
            }
        }
        //in_sol.copy_from_slice(&x);

        // 3(b) : Pour chaque ligne non couverte, ajouter la colonne valide de coût minimal
        for i in 0..instance.m {
            if covering_number[i] == 0 {
                let best_j = instance.rows[i].iter().find(|&&j| !eliminated[j]).unwrap();

                if sol_mark[*best_j] != sol_generation {
                    sol.push((*best_j, true));
                    sol_mark[*best_j] = sol_generation;

                    for &cov in &instance.covering_map[*best_j] {
                        covering_number[cov] += 1;
                    }
                }
            }
        }

        // 3(c) : Supprimer les colonnes redondantes en partant de l'indice le plus grand
        sol.sort_by(|a, b| b.0.cmp(&a.0));
        for (j_to_remove, validity) in sol.iter_mut() {
            // Tester si S - {j} reste réalisable (couvre toutes les lignes)
            let mut cannot = false;
            for (j, &kk) in instance.covering_map[*j_to_remove].iter().enumerate() {
                covering_number[kk] -= 1;
                if covering_number[kk] <= 0 {
                    cannot = true;
                    for (idx_jjj, &jjj) in instance.covering_map[*j_to_remove].iter().enumerate() {
                        if idx_jjj > j {
                            break;
                        }
                        covering_number[jjj] += 1;
                    }
                    break;
                }
            }
            if !cannot {
                *validity = false;
            }
        }

        let update2_time = start2.elapsed().as_millis();
        // 3(d) : Mettre à jour Z_UB
        let sol_cj: f64 = sol
            .iter()
            .map(|&f| if f.1 { instance.costs[f.0] } else { 0 })
            .sum::<i32>() as f64;
        z_ub = z_ub.min(sol_cj);

        // Etape 4
        if z_max.ceil() >= z_ub {
            break;
        }
        // Etape 5
        for k in 0..instance.n {
            if !eliminated[k] {
                if x[k] {
                    pk[k] = pk[k].max(z_lb);
                } else {
                    pk[k] = pk[k].max(z_lb + c_big[k]);
                }
                if pk[k] > z_ub {
                    eliminated[k] = true; // On marque comme éliminé sans toucher aux coûts
                }
            }
        }
        // Etape 6
        for i in 0..instance.m {
            let mut coverage = 0;

            for &j in &instance.rows[i] {
                if x[j] {
                    coverage += 1;
                }
            }
            g_big[i] = 1.0 - coverage as f64;

            if ti[i] == 0.0 && g_big[i] < 0.0 {
                g_big[i] = 0.0;
            }
        }
        // Etape 7
        let mut crutial_value = 0.;
        for i in 0..instance.m {
            crutial_value += g_big[i] * g_big[i];
        }
        if crutial_value == 0. {
            break;
        }
        // Etape 8
        if changed_zub >= 30 {
            f /= 2.;
        }
        let step_size_t = f * (1.05 * z_ub - z_lb) / crutial_value;
        // Etape 9
        if f <= 0.005 {
            //if iteration >= 1000 {
            break;
        }
        // Etape 10
        for i in 0..instance.m {
            ti[i] = (ti[i] + step_size_t * g_big[i]).max(0.);
        }

        iteration += 1;
        if z_max > last_zmax {
            changed_zub = 0;
            //f = 2.0;
            last_zmax = z_max;
        }
        changed_zub += 1;

        std::mem::swap(&mut x, &mut last_x);
        last_x.fill(false);
        for &(s, v) in sol.iter() {
            last_x[s] = v;
        }
        if debug {
            println!(
                "zub : {z_ub}, zmax : {z_max}, zlb : {z_lb}, f : {f} | {iteration} in {} ms | update2 {} us",
                start.elapsed().as_millis(),
                update2_time
            );
        }
    }
    println!("zub : {z_ub}, zmax : {z_max}, zlb : {z_lb}, f : {f} | {iteration}");
    return z_ub;
}
const FILE: [&str; 6] = [
    "../instances/scp41.txt",
    "../instances/scp510.txt",
    "../instances/scpnrh5.txt",
    "../instances/scpclr12.txt",
    "../instances/scpd5.txt",
    "../instances/scp61.txt",
];
fn main() {
    let start = Instant::now();
    let mut file_number: usize = 0;
    if env::args().len() > 1 {
        file_number = env::args().nth(1).unwrap().parse().unwrap();
    }
    let path = FILE[file_number];
    println!("Instances : {path}");
    let instance = parse_instance(path);
    println!("Parsed in {} ms", start.elapsed().as_millis());
    let start = Instant::now();
    solve(instance);
    println!("Computed in {} ms", start.elapsed().as_millis());
}
