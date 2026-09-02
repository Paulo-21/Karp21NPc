use std::fs;
use std::io::{self, Write};

/// Génère une instance Non-Unicost Set Cover.
///
/// Format de sortie :
/// m n
/// c1 c2 ... cn
/// k col1 col2 ... colk
/// ...
///
/// - m : nombre de lignes / éléments à couvrir
/// - n : nombre de colonnes / ensembles
/// - density : probabilité qu'une colonne couvre une ligne
/// - seed : graine du générateur pseudo-aléatoire
pub fn generate_instance(
    path: &str,
    m: usize,
    n: usize,
    density: f64,
    seed: u64,
) -> io::Result<()> {
    assert!(m > 0);
    assert!(n > 0);
    assert!((0.0..=1.0).contains(&density));

    let mut rng = SimpleRng::new(seed);

    // ------------------------------------------------------------
    // 1. Génération des coûts
    // ------------------------------------------------------------
    //
    // Coûts non uniformes.
    // On évite volontairement que toutes les colonnes aient le
    // même coût.
    //
    let mut costs = Vec::with_capacity(n);

    for _ in 0..n {
        let cost = rng.gen_range(1, 1000);
        costs.push(cost);
    }

    // ------------------------------------------------------------
    // 2. Génération de la couverture
    // ------------------------------------------------------------
    //
    // rows[i] contient les colonnes qui couvrent l'élément i.
    //
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); m];

    for i in 0..m {
        for col in 0..n {
            if rng.gen_f64() < density {
                rows[i].push(col + 1); // format 1-based
            }
        }

        // Chaque élément doit être couvert au moins une fois.
        if rows[i].is_empty() {
            let col = rng.gen_range(0, n);
            rows[i].push(col + 1);
        }
    }

    // ------------------------------------------------------------
    // 3. Écriture du fichier
    // ------------------------------------------------------------

    let mut file = fs::File::create(path)?;

    writeln!(file, "{} {}", m, n)?;

    // Coûts
    for (i, cost) in costs.iter().enumerate() {
        if i > 0 {
            write!(file, " ")?;
        }
        write!(file, "{}", cost)?;
    }
    writeln!(file)?;

    // Lignes de couverture
    for row in &rows {
        write!(file, "{}", row.len())?;

        for &col in row {
            write!(file, " {}", col)?;
        }

        writeln!(file)?;
    }

    Ok(())
}

/// Petit générateur pseudo-aléatoire déterministe.
/// Pas besoin de crate externe.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // LCG 64 bits
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        self.state
    }

    fn gen_f64(&mut self) -> f64 {
        let x = self.next_u64();

        // Valeur dans [0, 1)
        (x as f64) / (u64::MAX as f64)
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        assert!(min < max);

        min + (self.next_u64() as usize % (max - min))
    }
}
