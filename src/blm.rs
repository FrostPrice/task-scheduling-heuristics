use crate::animation::{AnimationEvent, MachineState};
use crate::utils::Result;
use rand::Rng;
use std::time::Instant;

pub struct Maquina {
    pub tarefas: Vec<u32>,
    pub pos: i32,
}

impl Maquina {
    pub fn new(n: usize) -> Self {
        Maquina {
            tarefas: vec![0; n],
            pos: -1,
        }
    }

    pub fn ms_maquina(&self) -> u32 {
        if self.pos < 0 {
            return 0;
        }
        self.tarefas[0..=(self.pos as usize)].iter().sum()
    }
}

pub fn ms_total(maquinas: &[Maquina]) -> u32 {
    maquinas.iter().map(|m| m.ms_maquina()).max().unwrap_or(0)
}

pub fn pos_ms_min(maquinas: &[Maquina]) -> usize {
    maquinas
        .iter()
        .enumerate()
        .min_by_key(|(_, m)| m.ms_maquina())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn search_max_value(maquina: &Maquina, filtrar_menor: u32) -> i32 {
    let mut pos = -1;
    let mut valor = 0;

    for i in 0..=(maquina.pos as usize) {
        if maquina.tarefas[i] > valor && (filtrar_menor == 0 || maquina.tarefas[i] < filtrar_menor)
        {
            valor = maquina.tarefas[i];
            pos = i as i32;
        }
    }
    pos
}

pub fn melhor_melhora(tam_m: usize, tam_n: usize, tam_r: f64) -> Result {
    melhor_melhora_with_events(tam_m, tam_n, tam_r, None)
}

pub fn melhor_melhora_with_events(
    tam_m: usize,
    tam_n: usize,
    tam_r: f64,
    mut event_collector: Option<&mut Vec<AnimationEvent>>,
) -> Result {
    let mut maquinas: Vec<Maquina> = (0..tam_m).map(|_| Maquina::new(tam_n)).collect();
    let mut rng = rand::thread_rng();

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
    let mut moves: usize = 0;

    loop {
        let ms = ms_total(&maquinas);
        let pos_min = pos_ms_min(&maquinas);

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::ComparingMachines {
                min_machine_id: pos_min,
                min_makespan: maquinas[pos_min].ms_maquina(),
                total_makespan: ms,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(&maquinas),
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
                        machines: MachineState::from_maquinas(&maquinas),
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
                machines: MachineState::from_maquinas(&maquinas),
            });
        }

        let old_makespan = ms;

        maquinas[pos_min].pos += 1;
        let pos = maquinas[pos_min].pos as usize;
        maquinas[pos_min].tarefas[pos] = tarefa;

        maquinas[0].tarefas.remove(pos_max_value as usize);
        maquinas[0].tarefas.push(0);
        maquinas[0].pos -= 1;
        moves += 1;

        let new_makespan = ms_total(&maquinas);

        if let Some(events) = event_collector.as_deref_mut() {
            events.push(AnimationEvent::TaskMoved {
                from_machine: 0,
                to_machine: pos_min,
                task_value: tarefa,
                old_makespan,
                new_makespan,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(&maquinas),
            });
            events.push(AnimationEvent::IterationComplete {
                iteration: moves as u32,
                total_makespan: new_makespan,
            });
            events.push(AnimationEvent::MachineSnapshot {
                machines: MachineState::from_maquinas(&maquinas),
            });
        }
    }

    let ms_f = ms_total(&maquinas);
    let tempo_exec = tempo_s.elapsed().as_secs_f64() * 1000.0;

    if let Some(events) = event_collector {
        events.push(AnimationEvent::AlgorithmComplete {
            final_makespan: ms_f,
            total_moves: moves as u32,
        });
        events.push(AnimationEvent::MachineSnapshot {
            machines: MachineState::from_maquinas(&maquinas),
        });
    }

    Result {
        n_tarefas: tam_n,
        n_maquinas: tam_m,
        replicacao: tam_r,
        tempo_exec,
        iteracoes: moves,
        makespan_inicial: ms_s,
        makespan_final: ms_f,
        algoritmo: "busca-local-monotona-melhorada".to_string(),
        perturbacao: 0.0,
    }
}
