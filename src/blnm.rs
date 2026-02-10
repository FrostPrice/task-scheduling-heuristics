use crate::animation::{AnimationEvent, MachineState};
use crate::blm::{ms_total, Maquina};
use crate::utils::Result;
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

fn perturbar(
    maquinas: &mut [Maquina],
    perturbacao: f64,
    mut event_collector: Option<&mut Vec<AnimationEvent>>,
) {
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

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::PerturbationMove {
                from_machine: idx_origem,
                to_machine: idx_destino,
                task_value: tarefa,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(maquinas),
            });
        }
    }
}

fn aplicar_busca_local(
    maquinas: &mut [Maquina],
    mut event_collector: Option<&mut Vec<AnimationEvent>>,
) {
    use crate::blm::{pos_ms_min, search_max_value};

    loop {
        let ms = ms_total(maquinas);
        let pos_min = pos_ms_min(maquinas);

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::ComparingMachines {
                min_machine_id: pos_min,
                min_makespan: maquinas[pos_min].ms_maquina(),
                total_makespan: ms,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(maquinas),
            });
        }

        if pos_min == 0 {
            break;
        }

        let ms_n = maquinas[pos_min].ms_maquina();
        let pos_max_value = search_max_value(&maquinas[0], 0);

        if pos_max_value == -1 || ms_n + maquinas[0].tarefas[pos_max_value as usize] > ms {
            if let Some(events) = event_collector.as_deref_mut() {
                if pos_max_value != -1 {
                    let task_value = maquinas[0].tarefas[pos_max_value as usize];
                    events.push(AnimationEvent::EvaluatingMove {
                        from_machine: 0,
                        to_machine: pos_min,
                        task_value,
                        task_index: pos_max_value as usize,
                        new_makespan_would_be: ms_n + task_value,
                        current_makespan: ms,
                        will_move: false,
                    });
                    events.push(AnimationEvent::MachineSnapshot {
                        machines: MachineState::from_maquinas(maquinas),
                    });
                }
            }
            break;
        }

        let tarefa = maquinas[0].tarefas[pos_max_value as usize];

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::EvaluatingMove {
                from_machine: 0,
                to_machine: pos_min,
                task_value: tarefa,
                task_index: pos_max_value as usize,
                new_makespan_would_be: ms_n + tarefa,
                current_makespan: ms,
                will_move: true,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(maquinas),
            });
        }

        let old_makespan = ms;

        maquinas[pos_min].pos += 1;
        let pos = maquinas[pos_min].pos as usize;
        maquinas[pos_min].tarefas[pos] = tarefa;

        maquinas[0].tarefas.remove(pos_max_value as usize);
        maquinas[0].tarefas.push(0);
        maquinas[0].pos -= 1;

        let new_makespan = ms_total(maquinas);

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::TaskMoved {
                from_machine: 0,
                to_machine: pos_min,
                task_value: tarefa,
                old_makespan,
                new_makespan,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(maquinas),
            });
        }
    }
}

pub fn busca_local_iterada(
    tam_m: usize,
    tam_n: usize,
    tam_r: f64,
    perturbacao: f64,
    max_iteracoes_sem_melhora: u32,
) -> Result {
    busca_local_iterada_with_events(
        tam_m,
        tam_n,
        tam_r,
        perturbacao,
        max_iteracoes_sem_melhora,
        None,
    )
}

pub fn busca_local_iterada_with_events(
    tam_m: usize,
    tam_n: usize,
    tam_r: f64,
    perturbacao: f64,
    max_iteracoes_sem_melhora: u32,
    mut event_collector: Option<&mut Vec<AnimationEvent>>,
) -> Result {
    let mut maquinas: Vec<Maquina> = (0..tam_m).map(|_| Maquina::new(tam_n)).collect();
    let mut rng = rand::thread_rng();

    // Gerar solução inicial aleatória
    for i in 0..tam_n {
        let value = rng.gen_range(1..=100);
        maquinas[0].tarefas[i] = value;
        maquinas[0].pos += 1;

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::TaskGenerated {
                machine_id: 0,
                task_value: value,
                task_index: i,
            });
        }
    }

    if let Some(events) = event_collector.as_deref_mut() {
        events.push(AnimationEvent::MachineSnapshot {
            machines: MachineState::from_maquinas(&maquinas),
        });
    }

    let ms_s = ms_total(&maquinas);
    let tempo_s = Instant::now();

    // Aplicar busca local na solução inicial
    let mut melhor_solucao = clonar_solucao(&maquinas);

    if let Some(events) = event_collector.as_deref_mut() {
        events.push(AnimationEvent::LocalSearchStart { iteration: 0 });
        events.push(AnimationEvent::MachineSnapshot {
            machines: MachineState::from_maquinas(&melhor_solucao),
        });
    }

    aplicar_busca_local(&mut melhor_solucao, event_collector.as_deref_mut());
    let mut melhor_makespan = ms_total(&melhor_solucao);

    let mut iteracoes_sem_melhora: u32 = 0;
    let mut iteracoes_totais: usize = 0;

    while iteracoes_sem_melhora < max_iteracoes_sem_melhora {
        iteracoes_totais += 1;

        // Perturbar a melhor solução
        let mut solucao_perturbada = clonar_solucao(&melhor_solucao);

        let total_tarefas: usize = solucao_perturbada
            .iter()
            .map(|m| if m.pos >= 0 { (m.pos + 1) as usize } else { 0 })
            .sum();
        let num_perturb = ((total_tarefas as f64) * perturbacao).max(1.0) as usize;

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::PerturbationStart {
                iteration: iteracoes_totais as u32,
                num_moves: num_perturb,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(&solucao_perturbada),
            });
        }

        perturbar(
            &mut solucao_perturbada,
            perturbacao,
            event_collector.as_deref_mut(),
        );

        // Aplicar busca local
        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::LocalSearchStart {
                iteration: iteracoes_totais as u32,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(&solucao_perturbada),
            });
        }

        aplicar_busca_local(&mut solucao_perturbada, event_collector.as_deref_mut());

        // Avaliar nova solução
        let makespan_atual = ms_total(&solucao_perturbada);

        // Aceitar se melhor
        if makespan_atual < melhor_makespan {
            let old_best = melhor_makespan;
            melhor_solucao = solucao_perturbada;
            melhor_makespan = makespan_atual;
            iteracoes_sem_melhora = 0;

            if let Some(events) = event_collector.as_deref_mut() {
                events.push(AnimationEvent::BestSolutionUpdated {
                    iteration: iteracoes_totais as u32,
                    old_best,
                    new_best: melhor_makespan,
                });
                events.push(AnimationEvent::MachineSnapshot {
                    machines: MachineState::from_maquinas(&melhor_solucao),
                });
            }
        } else {
            iteracoes_sem_melhora += 1;

            if let Some(events) = event_collector.as_deref_mut() {
                events.push(AnimationEvent::NoImprovement {
                    iteration: iteracoes_totais as u32,
                    stagnation_count: iteracoes_sem_melhora,
                });
                events.push(AnimationEvent::MachineSnapshot {
                    machines: MachineState::from_maquinas(&melhor_solucao),
                });
            }
        }
    }

    let tempo_exec = tempo_s.elapsed().as_secs_f64() * 1000.0;

    if let Some(events) = event_collector {
        events.push(AnimationEvent::AlgorithmComplete {
            final_makespan: melhor_makespan,
            total_moves: iteracoes_totais as u32,
        });
        events.push(AnimationEvent::MachineSnapshot {
            machines: MachineState::from_maquinas(&melhor_solucao),
        });
    }

    Result {
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
