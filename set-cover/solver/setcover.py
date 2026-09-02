from sys import argv

from ortools.sat.python import cp_model
import time

def parse_scp(filename):
    """
    Parse un fichier SCP de l'OR-Library.

    Format :
        m n
        c(1) c(2) ... c(n)
        k_1
        colonnes couvrant la ligne 1
        k_2
        colonnes couvrant la ligne 2
        ...

    m = nombre de lignes / contraintes
    n = nombre de colonnes / ensembles
    """
    print(f"Instance : {filename}")
    with open(filename, "r") as f:
        tokens = f.read().split()

    pos = 0

    # ---------------------------------------------------------
    # Header
    # ---------------------------------------------------------

    m = int(tokens[pos])       # nombre de lignes
    n = int(tokens[pos + 1])   # nombre de colonnes
    pos += 2

    # ---------------------------------------------------------
    # Coûts des n colonnes
    # ---------------------------------------------------------

    if pos + n > len(tokens):
        raise ValueError(
            f"Fichier trop court : impossible de lire les {n} coûts."
        )

    costs = list(map(int, tokens[pos:pos + n]))
    pos += n

    # ---------------------------------------------------------
    # Lignes / contraintes
    # ---------------------------------------------------------

    rows = []

    for i in range(m):

        if pos >= len(tokens):
            raise ValueError(
                f"Fichier terminé avant la ligne {i + 1}/{m}"
            )

        # Nombre de colonnes couvrant cette ligne
        k = int(tokens[pos])
        pos += 1

        if pos + k > len(tokens):
            available = len(tokens) - pos

            raise ValueError(
                f"Ligne {i + 1}: attendu {k} indices, "
                f"mais seulement {available} disponibles."
            )

        # Indices des colonnes
        indices = list(map(int, tokens[pos:pos + k]))
        pos += k

        # Vérification des indices
        for j in indices:
            if not 1 <= j <= n:
                raise ValueError(
                    f"Ligne {i + 1}: indice {j} hors limites "
                    f"(doit être entre 1 et {n})."
                )

        # Conversion 1-based -> 0-based
        rows.append([j - 1 for j in indices])

    # ---------------------------------------------------------
    # Vérifier qu'il ne reste rien
    # ---------------------------------------------------------

    if pos != len(tokens):
        raise ValueError(
            f"Données restantes après parsing : {len(tokens) - pos}"
        )

    return m, n, costs, rows


def solve(m, n, costs, rows):
    # ---------------------------------------------------------
    # Parsing
    # ---------------------------------------------------------




    print(f"Nombre de lignes    : {m}")
    print(f"Nombre de colonnes  : {n}")
    print(f"Nombre de coûts     : {len(costs)}")
    print(f"Nombre de contraintes : {len(rows)}")

    # ---------------------------------------------------------
    # Modèle CP-SAT
    # ---------------------------------------------------------

    model = cp_model.CpModel()

    # x[j] = 1 si la colonne / ensemble j est sélectionné
    #
    # IMPORTANT :
    # Il y a n colonnes, pas m.
    x = [
        model.new_bool_var(f"x_{j}")
        for j in range(n)
    ]

    # ---------------------------------------------------------
    # Chaque ligne doit être couverte au moins une fois
    # ---------------------------------------------------------

    for i in range(m):

        if not rows[i]:
            raise ValueError(
                f"La ligne {i + 1} n'est couverte par aucune colonne."
            )

        """model.Add(
            cp_model.LinearExpr.Sum() >= 1
        )"""
        model.add_bool_or(x[j] for j in rows[i])

    # ---------------------------------------------------------
    # Fonction objectif
    # ---------------------------------------------------------

    model.Minimize(
        sum(costs[j] * x[j] for j in range(n))
    )

    # ---------------------------------------------------------
    # CP-SAT
    # ---------------------------------------------------------

    solver = cp_model.CpSolver()

    solver.parameters.num_search_workers = 20
    solver.parameters.max_time_in_seconds = 6
    solver.parameters.log_search_progress = True
    solver.parameters.cp_model_presolve = False
    status = solver.Solve(model)

    # ---------------------------------------------------------
    # Résultat
    # ---------------------------------------------------------

    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):

        selected = [
            j + 1
            for j in range(n)
            if solver.Value(x[j])
        ]

        return {
            "status": solver.StatusName(status),
            "objective": solver.ObjectiveValue(),
            "selected_sets": selected,
        }

    return {
        "status": solver.StatusName(status),
        "objective": None,
        "selected_sets": [],
    }


if __name__ == "__main__":
    selected = 0
    if len(argv)>1:
        selected = int(argv[1])
    filenames = ["scp41.txt","scp510.txt","scp61.txt","scpclr12.txt","scpd5.txt","scpnrh5.txt"]
    #path = "../instances/"+filenames[selected]
    path = "instance.txt"
    m, n, costs, rows = parse_scp(path)
    t = time.time()
    result = solve(m, n, costs, rows)
    print("Solved in ",  time.time()-t)
    print()
    print("========================================")
    print("Résultat")
    print("========================================")
    print("Status              :", result["status"])
    print("Objectif            :", result["objective"])
    #print("Nombre de colonnes  :", len(result["selected_sets"]))
    #print("Colonnes sélectionnées :", result["selected_sets"])
