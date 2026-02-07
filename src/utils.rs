use crate::blm::BLMResult;
use std::fs::OpenOptions;
use std::io::{self, Write};

pub fn salvar_csv(resultado: &BLMResult, filename: &str) -> io::Result<()> {
    // Create results directory if it doesn't exist
    std::fs::create_dir_all("results")?;

    // Prepend results/ to the filename
    let filepath = format!("results/{filename}");

    let file_exists = std::path::Path::new(&filepath).exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filepath)?;

    // Write header if file is new
    if !file_exists {
        writeln!(
            file,
            "heuristica,n,m,replicacao,tempo,iteracoes,valor,parametro"
        )?;
    }

    // Write data: heuristica,n,m,replicacao,tempo,iteracoes,valor,parametro
    let parametro = if resultado.perturbacao > 0.0 {
        format!("{:.1}", resultado.perturbacao)
    } else {
        "NA".to_string()
    };
    writeln!(
        file,
        "{},{},{},{},{:.2},{},{},{}",
        resultado.algoritmo,
        resultado.n_tarefas,
        resultado.n_maquinas,
        resultado.replicacao,
        resultado.tempo_exec,
        resultado.iteracoes,
        resultado.makespan_final,
        parametro
    )?;

    Ok(())
}
