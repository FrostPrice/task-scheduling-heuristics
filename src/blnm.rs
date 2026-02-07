use crate::blm::{ms_total, BLMResult, Maquina};
use rand::Rng;
use std::time::Instant;

fn clonar_solucao(maquinas: &[Maquina]) -> Vec<Maquina> {
    maquinas
        .iter()
        .map(|m| Maquina {
            tarefas: m.tarefas.clone(),
            pos: m.pos,
        })
        .collect()
}

fn perturbar(maquinas: &mut [Maquina], perturbacao: f64) {
    let mut rng = rand::thread_rng();

    // Contar total de tarefas
    let total_tarefas: usize = maquinas
        .iter()
        .map(|m| if m.pos >= 0 { (m.pos + 1) as usize } else { 0 })
        .sum();

    if total_tarefas == 0 {
        return;
    }

    let num_perturb = ((total_tarefas as f64) * perturbacao).max(1.0) as usize;

    for _ in 0..num_perturb {
        // Encontrar máquina de origem aleatória que tenha tarefas
        let maquinas_com_tarefas: Vec<usize> = maquinas
            .iter()
            .enumerate()
            .filter(|(_, m)| m.pos >= 0)
            .map(|(i, _)| i)
            .collect();

        if maquinas_com_tarefas.is_empty() {
            break;
        }

        let idx_origem = maquinas_com_tarefas[rng.gen_range(0..maquinas_com_tarefas.len())];

        // Selecionar posição aleatória da tarefa
        let pos_tarefa = rng.gen_range(0..=(maquinas[idx_origem].pos as usize));
        let tarefa = maquinas[idx_origem].tarefas[pos_tarefa];

        // Selecionar máquina de destino diferente da origem
        let mut idx_destino = rng.gen_range(0..maquinas.len());
        while idx_destino == idx_origem && maquinas.len() > 1 {
            idx_destino = rng.gen_range(0..maquinas.len());
        }

        // Remover tarefa da máquina de origem
        maquinas[idx_origem].tarefas.remove(pos_tarefa);
        maquinas[idx_origem].tarefas.push(0);
        maquinas[idx_origem].pos -= 1;

        // Adicionar tarefa à máquina de destino
        maquinas[idx_destino].pos += 1;
        let pos = maquinas[idx_destino].pos as usize;
        maquinas[idx_destino].tarefas[pos] = tarefa;
    }
}

fn aplicar_busca_local(maquinas: &mut [Maquina]) {
    use crate::blm::{pos_ms_min, search_max_value};

    loop {
        let ms = ms_total(maquinas);
        let pos_min = pos_ms_min(maquinas);

        if pos_min == 0 {
            break;
        }

        let ms_n = maquinas[pos_min].ms_maquina();
        let pos_max_value = search_max_value(&maquinas[0], 0);

        if pos_max_value == -1 || ms_n + maquinas[0].tarefas[pos_max_value as usize] > ms {
            break;
        }

        let tarefa = maquinas[0].tarefas[pos_max_value as usize];
        maquinas[pos_min].pos += 1;
        let pos = maquinas[pos_min].pos as usize;
        maquinas[pos_min].tarefas[pos] = tarefa;

        maquinas[0].tarefas.remove(pos_max_value as usize);
        maquinas[0].tarefas.push(0);
        maquinas[0].pos -= 1;
    }
}

pub fn busca_local_iterada(
    tam_m: usize,
    tam_n: usize,
    tam_r: f64,
    perturbacao: f64,
    max_iteracoes_sem_melhora: u32,
) -> BLMResult {
    let mut maquinas: Vec<Maquina> = (0..tam_m).map(|_| Maquina::new(tam_n)).collect();
    let mut rng = rand::thread_rng();

    // Gerar solução inicial aleatória
    for i in 0..tam_n {
        let value = rng.gen_range(1..=100);
        maquinas[0].tarefas[i] = value;
        maquinas[0].pos += 1;
    }

    let ms_s = ms_total(&maquinas);
    let tempo_s = Instant::now();

    // Aplicar busca local na solução inicial
    let mut melhor_solucao = clonar_solucao(&maquinas);
    aplicar_busca_local(&mut melhor_solucao);
    let mut melhor_makespan = ms_total(&melhor_solucao);

    let mut iteracoes_sem_melhora = 0;
    let mut iteracoes_totais = 0;

    while iteracoes_sem_melhora < max_iteracoes_sem_melhora {
        // Perturbar a melhor solução
        let mut solucao_perturbada = clonar_solucao(&melhor_solucao);
        perturbar(&mut solucao_perturbada, perturbacao);

        // Aplicar busca local
        aplicar_busca_local(&mut solucao_perturbada);

        // Avaliar nova solução
        let makespan_atual = ms_total(&solucao_perturbada);

        // Aceitar se melhor
        if makespan_atual < melhor_makespan {
            melhor_solucao = solucao_perturbada;
            melhor_makespan = makespan_atual;
            iteracoes_sem_melhora = 0;
        } else {
            iteracoes_sem_melhora += 1;
        }

        iteracoes_totais += 1;
    }

    let tempo_exec = tempo_s.elapsed().as_secs_f64() * 1000.0;

    BLMResult {
        n_tarefas: tam_n,
        n_maquinas: tam_m,
        replicacao: tam_r,
        tempo_exec,
        iteracoes: iteracoes_totais,
        makespan_inicial: ms_s,
        makespan_final: melhor_makespan,
        algoritmo: "busca-local-iterada".to_string(),
        perturbacao,
    }
}
