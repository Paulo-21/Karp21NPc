use std::{cmp::Ordering, f32, fs, time::Instant};

#[derive(Debug)]
struct Instance {
    m: usize,              // nombre de lignes
    n: usize,              // nombre de colonnes
    costs: Vec<i32>,       // coût de chaque colonne
    rows: Vec<Vec<usize>>, // colonnes qui couvrent chaque ligne
    covered: Vec<u32>,
}

fn parse_instance() -> Instance {
    let file = fs::read_to_string("../instances/scp41.txt").unwrap();
    //let file = fs::read_to_string("../instances/scp510.txt").unwrap();
    //let file = fs::read_to_string("../instances/scpnrh5.txt").unwrap();

    // Tous les nombres du fichier, indépendamment des retours à la ligne.
    let mut tokens = file.split_ascii_whitespace();

    // Première ligne : m n
    let m: usize = tokens.next().unwrap().parse().unwrap();
    let n: usize = tokens.next().unwrap().parse().unwrap();

    println!("m = {m}, n = {n}");

    // Les n coûts des colonnes
    let mut costs = Vec::with_capacity(n);
    let mut covered: Vec<u32> = vec![0; n];

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
            covered[col - 1] += 1;
        }
        rows.push(covered_by);
    }
    for covered_by in rows.iter_mut() {
        covered_by.sort_by(|&a, &b| {
            costs[a]
                .cmp(&costs[b])
                .then_with(|| covered[b].cmp(&covered[a]))
        });
    }

    Instance {
        m,
        n,
        costs,
        rows,
        covered,
    }
}
fn init_ti(instance: &Instance) -> Vec<f32> {
    let mut ti = Vec::with_capacity(instance.m);
    for i in 0..instance.m {
        let mut min = i32::MAX;
        // rows[i] contient les colonnes qui couvrent la ligne i
        for &j in &instance.rows[i] {
            min = min.min(instance.costs[j]);
        }
        ti.push(min as f32);
    }
    ti
}
fn solve(mut instance: Instance) -> f32 {
    let start = Instant::now();
    let mut z_max = f32::NEG_INFINITY;
    let mut z_ub = f32::MAX;
    let mut z_lb = 0.;
    let mut pk: Vec<f32> = instance.costs.iter().map(|f| *f as f32).collect();
    let mut ti = init_ti(&instance);
    let mut x = vec![false; instance.n];
    let mut c_big = vec![0.0; instance.n];
    let mut g_big: Vec<f32> = vec![0.0; instance.m];
    let mut f = 2.0;
    let mut changed_zub = 0;
    let mut iteration = 0;
    let mut sol = Vec::with_capacity(instance.n);
    let mut last_zmax = z_max;
    let mut eliminated = vec![false; instance.n];
    println!("Init : {} ms", start.elapsed().as_millis());
    loop {
        let start = Instant::now();
        // etape 2
        /*for j in 0..instance.n {
            c_big[j] = instance.costs[j] as f32;
            for i in 0..instance.m {
                if instance.rows[i].contains(&j) {
                    c_big[j] -= ti[i];
                }
            }
            c_big[j] = c_big[j];
            x[j] = c_big[j] < 0.0;
        }*/
        for j in 0..instance.n {
            if eliminated[j] {
                c_big[j] = f32::MAX; // Ignorer cette colonne
                x[j] = false;
                continue;
            }
            c_big[j] = instance.costs[j] as f32;
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
        sol.clear();

        // 3(a) : Ajouter les colonnes où X_j = 1
        for (j, &xp) in x.iter().enumerate() {
            if xp {
                sol.push(j);
            }
        }

        // 3(b) : Pour chaque ligne non couverte, ajouter la colonne valide de coût minimal
        for i in 0..instance.m {
            // Vérifier si la ligne i est couverte par 'sol'
            let mut covered = false;
            for &j in &sol {
                if instance.rows[i].contains(&j) {
                    covered = true;
                    break;
                }
            }

            // Si la ligne n'est pas couverte, trouver la meilleure colonne disponible
            if !covered {
                let mut best_j = 0;
                for &j in instance.rows[i].iter() {
                    if !eliminated[j] {
                        best_j = j;
                        break;
                    }
                }
                sol.push(best_j);
            }
        }

        // 3(c) : Supprimer les colonnes redondantes en partant de l'indice le plus grand
        sol.sort_unstable_by(|a, b| b.cmp(a));
        let mut temp_sol = sol.clone();
        for &j_to_remove in &sol {
            // Tester si S - {j} reste réalisable (couvre toutes les lignes)
            let candidate_sol: Vec<usize> = temp_sol
                .iter()
                .cloned()
                .filter(|&j| j != j_to_remove)
                .collect();

            let mut all_covered = true;
            for i in 0..instance.m {
                let mut covered = false;
                for &j in &candidate_sol {
                    if instance.rows[i].contains(&j) {
                        covered = true;
                        break;
                    }
                }
                if !covered {
                    all_covered = false;
                    break;
                }
            }

            if all_covered {
                temp_sol = candidate_sol;
            }
        }
        sol = temp_sol;
        //println!("UPDATE 2 : {} ms", start2.elapsed().as_millis());*
        let update2_time = start2.elapsed().as_millis();
        // 3(d) : Mettre à jour Z_UB
        let sol_cj: f32 = sol.iter().map(|&f| instance.costs[f]).sum::<i32>() as f32;
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
            g_big[i] = 1.0 - coverage as f32;

            if ti[i] == 0.0 && g_big[i] < 0.0 {
                g_big[i] = 0.0;
            }
        }
        // Etape 7
        let mut crutial_value = 0.;
        for i in 0..instance.m {
            crutial_value += g_big[i].powi(2);
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

        /*println!(
            "zub : {z_ub}, zmax : {z_max}, zlb : {z_lb}, f : {f} | {iteration} in {} ms | update 2 {}",
            start.elapsed().as_millis(),
            update2_time
        );*/
    }
    println!("zub : {z_ub}, zmax : {z_max}, zlb : {z_lb}, f : {f} | {iteration}");
    return z_ub;
}
fn main() {
    let start = Instant::now();
    let instance = parse_instance();
    println!("Parsed in {} ms", start.elapsed().as_millis());
    let start = Instant::now();
    solve(instance);
    println!("Computed in {} ms", start.elapsed().as_millis());
}
