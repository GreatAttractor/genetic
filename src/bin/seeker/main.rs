//
// genetic - genetic programming experiments
// Copyright (c) 2019 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//
//
// File description:
//   Experiment: seeker. Breed a program that controls an agent moving on a square grid,
//   capable of navigating between specified start and end points.
//

extern crate genetic;
extern crate rand;
extern crate rand_xorshift;
extern crate rayon;

use genetic::utils;
use genetic::vm;
use rand::prelude::*;
use rayon::prelude::*;

// --------------- Tunable experiment parameters ---------------

/// Random number generator seed used for creating the initial population, test cases and running the evolution.
const RND_SEED: u64 = 2;

/// Size of the world (a square grid).
const WORLD_SIZE: u32 = 128;

const NUM_PROGRAMS: usize = 128;
const MIN_INITIAL_PROG_LEN: usize = 16;
const MAX_INITIAL_PROG_LEN: usize = 32;
const MAX_PROGRAM_LENGTH: usize = 1024;

/// Number of virtual machine data slots used by programs.
const NUM_PROG_DATA_SLOTS: usize = 4;

const NUM_TEST_CASES: usize = 32;

/// Max. number of evolution iterations (evolution stops earlier if a program that solves all the test cases emerges).
const MAX_NUM_ITERATIONS: usize = 16000;

/// Max. number of instructions executed for each program during its fitness evaluation.
const MAX_EXEC_INSTRUCTIONS: usize = 5000;

/// Fraction of population's best programs to use for breeding the new generation.
const BEST_PROG_FRACTION: f64 = 0.2;

/// Used instead of `BEST_PROG_FRACTION` when mitigating a fitness plateau.
const BEST_PROG_FRACTION_IN_PLATEAU: f64 = 0.5;

/// Min. length of program segment exchanged during recombination (crossover).
const MIN_CROSSOVER_SEG_LENGTH: usize = 6;

/// Max. length of program segment exchanged during recombination (crossover).
const MAX_CROSSOVER_SEG_LENGTH: usize = MAX_PROGRAM_LENGTH/4;

/// Probability that a program undergoes mutation during an evolution step.
const MUTATION_PROBABILITY: f64 = 0.2;

/// Used instead of `MUTATION_PROBABILITY` when mitigating a fitness plateau.
const MUTATION_PROBABILITY_IN_PLATEAU: f64 = 1.0;

/// Number of mutations per evolution step (if `MUTATION_PROBABILITY` was satisfied).
const NUM_MUTATIONS: usize = 3;

/// Used instead of `NUM_MUTATIONS` when mitigating a fitness plateau.
const NUM_MUTATIONS_IN_PLATEAU: usize = 16;

// ------------------------------------------------------------

// VM program outputs.
mod outputs {
    /// Add 1 to agent's X coord.
    pub const INC_X: i32 = 0;
    /// Subtract 1 from agent's X coord.
    pub const DEC_X: i32 = 1;
    /// Add 1 to agent's Y coord.
    pub const INC_Y: i32 = 2;
    /// Subtract 1 from agent's Y coord.
    pub const DEC_Y: i32 = 3;
}

/// VM program inputs.
mod inputs {
    /// Get agent's X coord.
    pub const POS_X: i32 = 0;
    /// Get agent's Y coord.
    pub const POS_Y: i32 = 1;
    /// Get target's X coord.
    pub const TARGET_X: i32 = 2;
    /// Get target's Y coord.
    pub const TARGET_Y: i32 = 3;
}

fn get_allowed_instructions() -> &'static [vm::OpCode] {
    &[vm::OpCode::SetI(0),
      vm::OpCode::SetI(1),
      vm::OpCode::SetI(2),
      vm::OpCode::SetI(3),
      vm::OpCode::SetI(4),
      vm::OpCode::Input(inputs::POS_X),
      vm::OpCode::Input(inputs::POS_Y),
      vm::OpCode::Input(inputs::TARGET_X),
      vm::OpCode::Input(inputs::TARGET_Y),
      vm::OpCode::Output(outputs::INC_X),
      vm::OpCode::Output(outputs::DEC_X),
      vm::OpCode::Output(outputs::INC_Y),
      vm::OpCode::Output(outputs::DEC_Y),
      vm::OpCode::ItoV,
      vm::OpCode::VtoI,
      vm::OpCode::IncV,
      vm::OpCode::DecV,
      vm::OpCode::IncI,
      vm::OpCode::DecI,
      vm::OpCode::Load,
      vm::OpCode::Store,
      vm::OpCode::Swap,
      vm::OpCode::EndGoTo,
      vm::OpCode::GoToIfP,
      vm::OpCode::JumpIfN,
      vm::OpCode::EndJump,
      vm::OpCode::IfP,
      vm::OpCode::IfN,
      vm::OpCode::Cmp,
      vm::OpCode::Add,
      vm::OpCode::Sub,
      vm::OpCode::Mul,
      vm::OpCode::Div,
      vm::OpCode::Abs,
      vm::OpCode::Neg,
      vm::OpCode::Nop]
}

/// Test case for evaluating program's fitness.
struct TestCase {
    // agent's starting position
    pub pos_x: i32,
    pub pos_y: i32,
    // target position
    pub target_x: i32,
    pub target_y: i32
}

fn generate_test_cases(count: usize, world_size: u32, rng: &mut rand_xorshift::XorShiftRng) -> Vec<TestCase> {
    let mut result = Vec::<TestCase>::with_capacity(count);
    for _ in 0..count {
        result.push(TestCase{
            pos_x: rng.gen_range(0, world_size) as i32,
            pos_y: rng.gen_range(0, world_size) as i32,
            target_x: rng.gen_range(0, world_size) as i32,
            target_y: rng.gen_range(0, world_size) as i32
        });
    }

    result
}

fn generate_initial_population(rng: &mut rand_xorshift::XorShiftRng) -> utils::SortedEvaluatedPrograms {
    let programs = utils::generate_random_programs(
        NUM_PROGRAMS,
        MIN_INITIAL_PROG_LEN,
        MAX_INITIAL_PROG_LEN,
        NUM_PROG_DATA_SLOTS,
        get_allowed_instructions(),
        None,
        rng);

    utils::SortedEvaluatedPrograms::new(programs, vec![utils::WORST_FITNESS; NUM_PROGRAMS])
}

/// Evaluates genetic program's fitness.
///
/// Programs are used to control an agent moving on a square grid. The goal is to move
/// towards the target and stay around it as close as possible, ideally - reaching the target.
///
/// Reading from inputs returns the coordinates:
///     0 - agent.x
///     1 - agent.y
///     2 - target.x
///     3 - target.y
///
/// Writing to outputs (`reg_v` value is irrelevant) determines agent actions:
///     0 - increment agent.x by 1
///     1 - decrement agent.x by 1
///     2 - increment agent.y by 1
///     3 - decrement agent.y by 1
///
/// Returns (fitness, whether the program reached the target).
///
fn evaluate_fitness(
    program: &vm::Program,
    test_case: &TestCase
) -> (utils::Fitness, bool) {

    macro_rules! sqr{ ($x:expr) => { ($x) * ($x) }; }

    struct Agent {
        // current position
        pub x: i32,
        pub y: i32,
        // target position
        pub tx: i32,
        pub ty: i32,
        pub distance_travelled: i32
    }

    impl vm::InputOutputHandler for Agent {
        fn input(&mut self, input_num: i32) -> vm::RegValue {
            match input_num {
                inputs::POS_X => self.x as vm::RegValue,
                inputs::POS_Y => self.y as vm::RegValue,
                inputs::TARGET_X => self.tx as vm::RegValue,
                inputs::TARGET_Y => self.ty as vm::RegValue,
                _ => 0.0
            }
        }

        fn output(&mut self, output_num: i32, _output_val: vm::RegValue) {
            let old_x = self.x;
            let old_y = self.y;

            match output_num {
                outputs::INC_X => if self.x < WORLD_SIZE as i32 - 1 { self.x += 1; },
                outputs::DEC_X => if self.x > 0 { self.x -= 1; },
                outputs::INC_Y => if self.y < WORLD_SIZE as i32 - 1 { self.y += 1; },
                outputs::DEC_Y => if self.y > 0 { self.y -= 1; },
                _ => ()
            }

            if self.x != old_x || self.y != old_y {
                self.distance_travelled += 1;
            }
        }

        fn check_end_condition(&self, _num_execd_instructions: usize) -> bool {
            self.x == self.tx && self.y == self.ty
        }
    }

    let mut agent = Agent{
        x: test_case.pos_x,
        y: test_case.pos_y,
        tx: test_case.target_x,
        ty: test_case.target_y,
        distance_travelled: 0
    };

    {
        let opt_program = program.get_optimized();
        let mut vm = vm::VirtualMachine::new(&opt_program, Some(&mut agent));
        vm.run(Some(MAX_EXEC_INSTRUCTIONS), true, true);
    }

    let final_dist = f64::sqrt(sqr!(agent.x - agent.tx) as f64 + sqr!(agent.y - agent.ty) as f64);
    let reached_target = final_dist == 0.0;

    // fitness penalty for taking too long to reach the target
    let mut penalty = 1.0;

    if reached_target {
        // reduce the penalty if the program used a shorter path
        penalty = penalty * (1.0 - f64::exp(-1.0*agent.distance_travelled as f64));
    }

    return (penalty + final_dist, reached_target)
}

///
/// Evaluates fitness of `programs`.
///
/// Returns list of evaluated programs (sorted by fitness) and a flag indicating if any program solved all test cases.
///
fn evaluate_programs(programs: Vec<vm::Program>, test_cases: &[TestCase]) -> (utils::SortedEvaluatedPrograms, bool) {
    // fitness of each program
    let mut fitness = vec![0.0; programs.len()];
    // indicates if any program reached all targets
    let all_targets_reached = std::sync::atomic::AtomicBool::new(false);

    // runs in parallel using `RAYON_NUM_THREADS` CPU cores
    fitness.par_iter_mut().enumerate().for_each(
        |(i, f)| {
            let mut prog_fitness = 0.0;
            let mut prog_all_targets_reached = true;
            for test_case in test_cases.iter() {
                let (tcase_fitness, tcase_target_reached) = evaluate_fitness(&programs[i], test_case);
                prog_fitness += tcase_fitness;
                prog_all_targets_reached = prog_all_targets_reached && tcase_target_reached;
            }
            *f = prog_fitness;
            all_targets_reached.fetch_or(prog_all_targets_reached, std::sync::atomic::Ordering::Relaxed);
        }
    );

    (utils::SortedEvaluatedPrograms::new(programs, fitness), all_targets_reached.into_inner())
}

struct EvolutionState {
    pub mutation_probability: f64,
    pub best_prog_fraction: f64,
    pub num_mutations: usize,

    pub mitigating_plateau: bool,
    pub mitigation_step: usize,
    pub plateau_steps: usize,
    pub best_fitness: utils::Fitness
}

impl EvolutionState {
    const NUM_PLATEAU_MITIGATION_STEPS: usize = 30;
    const NUM_PLATEAU_DETECTION_STEPS: usize = 16;

    pub fn end_plateau_mitigation(&mut self) {
        self.mitigating_plateau = false;
        self.mutation_probability = MUTATION_PROBABILITY;
        self.num_mutations = NUM_MUTATIONS;
        self.best_prog_fraction = BEST_PROG_FRACTION;

        self.plateau_steps = 0;
    }

    pub fn enable_plateau_mitigation(&mut self) {
        self.mitigating_plateau = true;
        self.mutation_probability = MUTATION_PROBABILITY_IN_PLATEAU;
        self.num_mutations = NUM_MUTATIONS_IN_PLATEAU;
        self.best_prog_fraction = BEST_PROG_FRACTION_IN_PLATEAU;

        self.mitigation_step = 0;
        self.plateau_steps = 0;
    }
}

/// Returns new population of programs and a flag indicating if any current program solved all test cases.
fn evaluate_and_reproduce_best_programs(
    programs: utils::SortedEvaluatedPrograms,
    test_cases: &[TestCase],
    evolution: &mut EvolutionState,
    rng: &mut rand_xorshift::XorShiftRng
) -> (utils::SortedEvaluatedPrograms, bool) {
    //
    // 1) Create new population (of the same size as 'programs')
    //    by recombining and mutating a fraction of the best 'programs'.
    //
    let new_population = utils::create_new_population(
        programs,

        evolution.mutation_probability,
        evolution.num_mutations,
        evolution.best_prog_fraction,
        get_allowed_instructions(),
        MIN_CROSSOVER_SEG_LENGTH,
        MAX_CROSSOVER_SEG_LENGTH,
        MAX_PROGRAM_LENGTH,
        NUM_PROG_DATA_SLOTS,
        rng);

    //
    // 2) Evaluate fitness of the new population by running the programs for all test cases.
    //
    let (sorted_new_programs, all_targets_reached) = evaluate_programs(new_population, &test_cases);

    //
    // 3) Print statistics and mitigate a plateau if needed.
    //
    let best_fitness = sorted_new_programs.get_programs()[0].fitness;

    if best_fitness < evolution.best_fitness {
        evolution.best_fitness = best_fitness;
    }

    if evolution.mitigating_plateau {
        if evolution.mitigation_step < EvolutionState::NUM_PLATEAU_MITIGATION_STEPS {
            print!("(p) ");
            evolution.mitigation_step += 1;
        }
        else {
            evolution.end_plateau_mitigation();
        }
    } else {
        if best_fitness >= evolution.best_fitness {
            evolution.plateau_steps += 1;
        } else {
            evolution.plateau_steps = 0;
        }

        // if we reached a fitness plateau, temporarily speed up the evolution
        if evolution.plateau_steps > EvolutionState::NUM_PLATEAU_DETECTION_STEPS {
            print!("(p) ");
            evolution.enable_plateau_mitigation();
        }
    }

    println!("best fitness: {:.2} (so far: {:.2})", best_fitness, evolution.best_fitness);

    (sorted_new_programs, all_targets_reached)
}

fn main() {
    let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(RND_SEED);

    let mut evolution = EvolutionState{
        mutation_probability: MUTATION_PROBABILITY,
        best_prog_fraction: BEST_PROG_FRACTION,
        num_mutations: NUM_MUTATIONS,

        mitigating_plateau: false,
        mitigation_step: 0,
        plateau_steps: 0,
        best_fitness: utils::WORST_FITNESS
    };

    let test_cases = generate_test_cases(NUM_TEST_CASES, WORLD_SIZE, &mut rng);

    let mut programs = generate_initial_population(&mut rng);

    for i in 0..MAX_NUM_ITERATIONS {
        print!("{}: ", i);

        let (new_programs, all_targets_reached) = evaluate_and_reproduce_best_programs(programs, &test_cases, &mut evolution, &mut rng);
        if all_targets_reached {
            let optimized_best_prog = new_programs.get_programs()[0].prog.get_optimized();

            let output_vmasm = "program.vmasm";
            let output_jsvm = "src/bin/seeker/demo/program.js";
            println!("\nSaving the best program as:\n  \
                          - {} (VM assembly)\n  \
                          - {} (JavaScript virtual machine)", output_vmasm, output_jsvm);
            std::fs::write(
                output_vmasm,
                utils::pretty_print(
                    &optimized_best_prog,
                    Some("*"),
                    false,
                    Some(2)
                )
            ).expect(&format!("Could not write to {}.", output_vmasm));

            std::fs::write(
                output_jsvm,
                genetic::transpile::javascript_vm::program_to_javascript_vm(&optimized_best_prog)
            ).expect(&format!("Could not write to {}.", output_jsvm));

            break;
        }

        programs = new_programs;
    }
}
