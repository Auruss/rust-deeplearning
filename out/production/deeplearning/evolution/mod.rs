use std::vec::*;

/// Trait needs to be implemented when struct is used for an evoltionary algorithm
pub trait Evolvable {
    /// Crosses indivduals together
    fn cross_over(&self, other: &Self) -> Self;

    /// Slightly mutates self
    fn mutate(&mut self);
}

pub enum StopRule {
    /// Stops evolving when given fitness is reached
    FitnessReached(f64),
    /// Stops when generation x is reached
    GenerationReached(usize),
    /// Stops when when the fitness improvment stayed at 0 for x generations
    HasNotImprovedSince(usize),
    /// Stops never, runs infinitely
    Never
}

pub struct EvolutionOptions {
    /// Sets the amount of threads that should be used (to calculate fitness)
    ///   Defaults to the amount of cpu cores detected
    pub threads: usize,

    //pub hooks: Vec<EvolutionHooks>
}

impl EvolutionOptions {
    pub fn defaults() -> Self {
        // TODO: Detect cpu core amount
        EvolutionOptions {
            threads: 6
        }
    }
}

pub fn genetic_evolution<T: Evolvable + Clone + Sync + Send, Fnew, Frate>(population: usize, stop_rule: StopRule, new: Fnew, rate: &mut Frate, opt_options: Option<EvolutionOptions>) -> T
    where Fnew: Fn() -> T, Frate: FnMut(&T) -> f64
{
    // get options of grab defaults
    let options = match opt_options {
        Some (x) => x,
        _ => EvolutionOptions::defaults()
    };

    let mut generation: Vec<Box<T>> = Vec::new();
    let mut generationNo: usize = 0;
    let mut prev_fitness: f64 = 0.0;

    // create initial population
    for _ in 0..population {
        generation.push(Box::new(new()));
    }

    loop {
        let mut bests: Vec<(f64, usize)> = vec!((-9999.0, 0), (-9999.0, 0));

        // TODO how can we use a threadpool to calculate fitness?

        // Get two best individuals
        for i in 0..population {
            let fitness = rate(&generation[i]);

            if fitness > bests[0].0 {
                bests[1] = bests[0];
                bests[0] = (fitness, i);
            }
            else if  fitness > bests[1].0 {
                bests[1] = (fitness, i);
            }
        }

        info!(target: "genetic_evolution", "Highest Fitness in Generation {} equals {}, thats an improvment of {}", generationNo, bests[0].0, bests[0].0 - prev_fitness);
        prev_fitness = bests[0].0;

        // Stop Rule
        match stop_rule {
            StopRule::FitnessReached(fitness) => { },
            StopRule::GenerationReached(gen) => {
                if generationNo >= gen {
                    return (*generation[bests[0].1]).clone();
                }
            },
            StopRule::HasNotImprovedSince(gen) => { },
            StopRule::Never => { }
        }

        for i in (0..population).rev() {
            if i != bests[0].1 && i != bests[1].1 {
                generation.remove(i);
            }
        }

        // Create next Generation (2 best will continue to live)
        for _ in 2..population {
            // Create child from the two best
            let mut individual = generation[0].cross_over(&generation[1]);

            // Mutate
            individual.mutate();

            // Put into new generation
            generation.push(Box::new(individual));
        }

        // End generation
        generationNo += 1;
    }
}
