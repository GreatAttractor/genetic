//
// genetic - genetic programming experiments
// Copyright (c) 2019 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//
//
// File description:
//   Module: linear genetic programming utilities.
//

use rand::prelude::*;
use vm;

/// Represents fitness of a genetic program; lower values are better.
pub type Fitness = f64;

pub const WORST_FITNESS: Fitness = 99.0e+19;

pub struct EvaluatedProgram {
    pub fitness: Fitness,
    pub prog: vm::Program
}

/// List of evaluated programs sorted (ascending) by fitness.
pub struct SortedEvaluatedPrograms {
    programs: Vec<EvaluatedProgram>
}

impl SortedEvaluatedPrograms {
    /// Creates a list containing `programs` and `fitness` sorted (ascending) by fitness.
    pub fn new(programs: Vec<vm::Program>, fitness: Vec<Fitness>) -> SortedEvaluatedPrograms {
        assert!(programs.len() == fitness.len());
        let mut sorted_programs: Vec<EvaluatedProgram> = vec![];
        for (prog, fitness) in programs.into_iter().zip(fitness.into_iter()) {
            sorted_programs.push(EvaluatedProgram{ fitness, prog });
        }
        sorted_programs.sort();

        SortedEvaluatedPrograms{ programs: sorted_programs }
    }

    pub fn len(&self) -> usize { self.programs.len() }

    pub fn get_programs(&self) -> &[EvaluatedProgram] { &self.programs }
}

impl std::cmp::PartialEq for EvaluatedProgram {
    fn eq(&self, other: &EvaluatedProgram) -> bool {
        self.fitness == other.fitness
    }
}

impl Eq for EvaluatedProgram { }

impl std::cmp::PartialOrd for EvaluatedProgram {
    fn partial_cmp(&self, other: &EvaluatedProgram) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

impl Ord for EvaluatedProgram {
    fn cmp(&self, other: &EvaluatedProgram) -> std::cmp::Ordering {
        self.fitness.partial_cmp(&other.fitness).unwrap()
    }
}


///
/// Returns textual representation of program.
///
/// # Parameters
///
/// * `program` - The program to print.
/// * `inactive_jumps_marker` - If `Some`, wil be used to mark inactive
/// `GoToIfP`, `EndGoTo`, `JumpIfN`, `EndJump` instructions.
/// * `instr_numbers` - If true, print instruction numbers.
/// * `indentation_width` - Number of spaces per indendation level.
///
pub fn pretty_print(
    program: &vm::Program,
    inactive_jumps_marker: Option<&str>,
    instr_numbers: bool,
    indentation_width: Option<usize>
) -> String {
    let mut output = String::new();
    if program.get_instr().is_empty() {
        return output;
    }

    let inactive = inactive_jumps_marker.unwrap_or("");
    let jmp_tbl = program.get_jump_table();
    let indent = indentation_width.is_some();
    let mut indent_level = 0;

    // make sure the instruction numbers have enough space on the line
    let instr_num_width = 1 + f64::trunc(f64::log10(program.get_instr().len() as f64)) as usize;

    let mut prev_opcode = *program.get_instr().last().unwrap();

    for (i, opcode) in program.get_instr().iter().enumerate() {
        if instr_numbers {
            output += &format!("{:1$} ", i, instr_num_width);
        }

        if jmp_tbl[i].is_some() && (*opcode == vm::OpCode::GoToIfP || *opcode == vm::OpCode::EndJump) {
            indent_level -= 1;
        }

        if indent {
            // additional identation (only for the current instruction) if the previous opcode was `IfP` or `IfN`
            let mut actual_lvl = indent_level + if prev_opcode == vm::OpCode::IfN || prev_opcode == vm::OpCode::IfP { 1 } else { 0 };
            output += &" ".repeat(actual_lvl * indentation_width.unwrap());
        }

        if jmp_tbl[i].is_some() && (*opcode == vm::OpCode::EndGoTo || *opcode == vm::OpCode::JumpIfN) {
            indent_level += 1;
        }

        let instr_mnemonic;
        match opcode {
            vm::OpCode::SetI(i) =>   instr_mnemonic = format!("seti {}", i),
            vm::OpCode::Input(i) =>  instr_mnemonic = format!("input {}", i),
            vm::OpCode::Output(i) => instr_mnemonic = format!("output {}", i),
            vm::OpCode::ItoV =>      instr_mnemonic = "itov".to_string(),
            vm::OpCode::VtoI =>      instr_mnemonic = "vtoi".to_string(),
            vm::OpCode::IncV =>      instr_mnemonic = "incv".to_string(),
            vm::OpCode::DecV =>      instr_mnemonic = "decv".to_string(),
            vm::OpCode::IncI =>      instr_mnemonic = "inci".to_string(),
            vm::OpCode::DecI =>      instr_mnemonic = "deci".to_string(),
            vm::OpCode::Load =>      instr_mnemonic = "load".to_string(),
            vm::OpCode::Store =>     instr_mnemonic = "store".to_string(),
            vm::OpCode::Swap =>      instr_mnemonic = "swap".to_string(),
            vm::OpCode::EndGoTo =>   instr_mnemonic = "endgoto".to_string(),
            vm::OpCode::GoToIfP =>   instr_mnemonic = "gotoifp".to_string(),
            vm::OpCode::JumpIfN =>   instr_mnemonic = "jumpifn".to_string(),
            vm::OpCode::EndJump =>   instr_mnemonic = "endjump".to_string(),
            vm::OpCode::IfP =>       instr_mnemonic = "ifp".to_string(),
            vm::OpCode::IfN =>       instr_mnemonic = "ifn".to_string(),
            vm::OpCode::Cmp =>       instr_mnemonic = "cmp".to_string(),
            vm::OpCode::Add =>       instr_mnemonic = "add".to_string(),
            vm::OpCode::Sub =>       instr_mnemonic = "sub".to_string(),
            vm::OpCode::Mul =>       instr_mnemonic = "mul".to_string(),
            vm::OpCode::Div =>       instr_mnemonic = "div".to_string(),
            vm::OpCode::Abs =>       instr_mnemonic = "abs".to_string(),
            vm::OpCode::Neg =>       instr_mnemonic = "neg".to_string(),
            vm::OpCode::Sqrt =>      instr_mnemonic = "sqrt".to_string(),
            vm::OpCode::Nop =>       instr_mnemonic = "nop".to_string()
        }

        if jmp_tbl[i].is_none() &&
           (*opcode == vm::OpCode::EndGoTo ||
            *opcode == vm::OpCode::EndJump ||
            *opcode == vm::OpCode::GoToIfP ||
            *opcode == vm::OpCode::JumpIfN) {
                output += inactive;
        }

        output += &format!("{}\n", instr_mnemonic);

        prev_opcode = *opcode;
    }
    output
}

///
/// Generates a set of random programs.
///
/// # Parameters
///
/// * `num_programs` - Number of programs to generate.
/// * `min_length` - Min. program length.
/// * `max_length` - Max. program length.
/// * `num_data_slots` - Number of virtual machine data slots each program will use.
/// * `allowed_instructions` - List of allowed instructions.
/// * `rel_probability` - Relative probability of each instruction in `allowed_instructions`.
/// If `None`, each instruction is equally probable.
/// * `rng` - Random number generator to use.
///
pub fn generate_random_programs(
    num_programs: usize,
    min_length: usize,
    max_length: usize,
    num_data_slots: usize,
    allowed_instructions: &[vm::OpCode],
    rel_probability: Option<&[f64]>,
    rng: &mut rand_xorshift::XorShiftRng)
-> Vec<vm::Program> {
    assert!(min_length > 0 && max_length >= min_length);
    assert!(allowed_instructions.len() > 0);
    if rel_probability.is_some() {
        assert!(allowed_instructions.len() == rel_probability.unwrap().len());
    }

    let mut cumulative_probability = vec![0.0];
    let mut current_cumulative = 0.0;

    {
        let mut prob_adder = |p: f64| {
            current_cumulative += p;
            cumulative_probability.push(current_cumulative);
        };

        if rel_probability.is_some() {
            for p in rel_probability.unwrap() {
                prob_adder(*p);
            }
        } else {
            for _ in 0..allowed_instructions.len() {
                prob_adder(1.0);
            }
        }
    }
    let rel_prob_sum = current_cumulative;

    let mut result = vec![];

    for _ in 0..num_programs {
        let mut instructions = vec![];

        let prog_len = rng.gen_range(min_length, max_length + 1);

        for _ in 0..prog_len {
            let f: f64 = rng.gen_range(0.0, rel_prob_sum);

            let mut opcode_loc;
            match cumulative_probability.binary_search_by(|x| x.partial_cmp(&f).unwrap()) {
                Ok(x) => opcode_loc = x,
                Err(x) => opcode_loc = x - 1
            }

            instructions.push(allowed_instructions[opcode_loc]);
        }

        result.push(vm::Program::new(&instructions, num_data_slots, false));
    }

    result
}

/// Returns the greatest length (up to `length`) of a code segment from `start` which does not cross a control flow block boundary.
fn limit_length_to_not_crossing(program: &[vm::OpCode], start: usize, length: usize) -> usize {
    let mut result = length;

    let mut jump_level = 0;
    let mut goto_level = 0;

    let mut pos = start;
    let mut last_zero_pos = pos; // last position where both levels were zero

    while pos < start + length {
        match program[pos] {
            vm::OpCode::EndGoTo => goto_level += 1,
            vm::OpCode::GoToIfP => if goto_level > 0 { goto_level -= 1; }, // if already zero, this is an unmatched (ineffective) instruction
            vm::OpCode::JumpIfN => jump_level += 1,
            vm::OpCode::EndJump => if jump_level > 0 { jump_level -= 1; }, // if already zero, this is an unmatched (ineffective) instruction
            _ => ()
        }

        if goto_level == 0 && jump_level == 0 {
            last_zero_pos = pos;
        }
        pos += 1;
    }

    // if specified `length` crosses a block, limit it to the last non-crossing position
    if jump_level != 0 || goto_level != 0 {
        result = last_zero_pos - start;
    }

    result
}

///
/// Exchanges randomly chosen segments between programs.
///
/// For each program, a randomly positioned and sized segment is chosen,
/// and swapped with the segment in the other program.
///
/// # Parameters
///
/// * `prog1` - First program to recombine.
/// * `prog2` - Second program to recombine.
/// * `min_seg_len` - Min. segment length.
/// * `max_seg_len` - Max. segment length.
/// * `allow_control_flow_block_xing` - If true, segments are allowed to cross control flow blocks
/// (`GoToIfP`/`EndGoTo` and `JumpIfN`/`EndJump` pairs).
/// * `rng` - Random number generator to use.
///
pub fn recombine_programs(
    prog1: &mut Vec<vm::OpCode>,
    prog2: &mut Vec<vm::OpCode>,
    min_seg_len: usize,
    max_seg_len: usize,
    allow_control_flow_block_xing: bool,
    rng: &mut rand_xorshift::XorShiftRng
) {
    assert!(max_seg_len >= min_seg_len);

    let exchg_pos_1: usize = rng.gen_range(0, prog1.len());
    let mut exchg_len_1: usize = std::cmp::min(rng.gen_range(min_seg_len, max_seg_len + 1), prog1.len() - exchg_pos_1);

    let exchg_pos_2: usize = rng.gen_range(0, prog2.len());
    let mut exchg_len_2: usize = std::cmp::min(rng.gen_range(min_seg_len, max_seg_len + 1), prog2.len() - exchg_pos_2);

    if !allow_control_flow_block_xing {
        exchg_len_1 = limit_length_to_not_crossing(prog1, exchg_pos_1, exchg_len_1);
        exchg_len_2 = limit_length_to_not_crossing(prog2, exchg_pos_2, exchg_len_2);
    }

    let mut new_prog1: Vec<vm::OpCode> = vec![];
    let mut new_prog2: Vec<vm::OpCode> = vec![];

    new_prog1.extend(prog1[0..exchg_pos_1].iter());
    new_prog1.extend(prog2[exchg_pos_2 .. exchg_pos_2 + exchg_len_2].iter());
    new_prog1.extend(prog1[exchg_pos_1 + exchg_len_1 ..].iter());

    new_prog2.extend(prog2[0..exchg_pos_2].iter());
    new_prog2.extend(prog1[exchg_pos_1 .. exchg_pos_1 + exchg_len_1].iter());
    new_prog2.extend(prog2[exchg_pos_2 + exchg_len_2 ..].iter());

    *prog1 = new_prog1;
    *prog2 = new_prog2;
}

pub fn mutate(
    program: &mut Vec<vm::OpCode>,
    num_mutations: usize,
    allowed_instructions: &[vm::OpCode],
    rng: &mut rand_xorshift::XorShiftRng
) {
    if program.len() == 0 { return; }

    let actual_num_mutations: usize = rng.gen_range(1, num_mutations+1);

    for _ in 0..actual_num_mutations {
        let f: f64 = rng.gen(); // selector of mutation type

        let mut pos: usize = rng.gen_range(0, program.len());

        let new_opcode = allowed_instructions[rng.gen_range(0, allowed_instructions.len())];

        if f < 1.0/4.0 {
            // insertion
            program.insert(pos, new_opcode);
        } else if f < 2.0/4.0 && program.len() > 1 {
            // deletion
            program.remove(pos);
        } else if f < 3.0/4.0 {
            // substitution
            program[pos] = new_opcode;
        } else if program.len() >= 2 {
            // transposition
            if pos == 0 { pos = 1 };
            program.swap(pos, pos - 1);
        }
    }
}

/// Returns a new population created by recombining and mutating the best of `programs`.
pub fn create_new_population(
    programs: SortedEvaluatedPrograms,
    mutation_probability: f64,
    num_mutations: usize,
    best_prog_fraction: f64,
    allowed_instructions: &[vm::OpCode],
    min_crossover_seg_length: usize,
    max_crossover_seg_length: usize,
    max_program_length: usize,
    num_program_data_slots: usize,
    rng: &mut rand_xorshift::XorShiftRng
) -> Vec<vm::Program> {
    let num_best_programs = (programs.len() as f64 * best_prog_fraction) as usize;
    let best_programs: Vec<&EvaluatedProgram> = programs.get_programs().iter().take(num_best_programs).collect();

    let mut new_population: Vec<vm::Program> = vec![];

    for _ in 0 .. programs.len()/2 {

        let index1: usize = rng.gen_range(0, best_programs.len());
        let index2: usize = rng.gen_range(0, best_programs.len());

        let mut prog1 = vec![]; prog1.extend_from_slice(best_programs[index1].prog.get_instr());
        let mut prog2 = vec![]; prog2.extend_from_slice(best_programs[index2].prog.get_instr());

        recombine_programs(&mut prog1, &mut prog2, min_crossover_seg_length, max_crossover_seg_length, true, rng);

        if prog1.len() > max_program_length {
            prog1.truncate(max_program_length);
        }
        if prog2.len() > max_program_length {
            prog2.truncate(max_program_length);
        }

        if rng.gen::<f64>() <= mutation_probability {
            mutate(&mut prog1, num_mutations, allowed_instructions, rng);
        }

        if rng.gen::<f64>() <= mutation_probability {
            mutate(&mut prog2, num_mutations, allowed_instructions, rng);
        }

        new_population.push(vm::Program::new(&prog1, num_program_data_slots, true));
        new_population.push(vm::Program::new(&prog2, num_program_data_slots, true));
    }

    // if the number of programs is odd, just copy one of the best ones without recombining
    if programs.len() % 2 == 1 {
        new_population.push(best_programs[rng.gen_range(0, best_programs.len())].prog.clone());
    }

    new_population
}
