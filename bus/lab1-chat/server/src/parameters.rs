use primes::PrimeSet;
use rand;

pub fn generate_parameters() -> (u32, u32) {
    let p = generate_p();
    let primitive_roots = find_primitive_roots(p);
    let idx = (rand::random::<f32>() * (primitive_roots.len() as f32 - 0.1)) as usize;
    let g = primitive_roots[idx];
    (p as u32, g)
}

fn find_primitive_roots(p: u64) -> Vec<u32> {
    let prime_factors = find_prime_factors(p - 1);
    let mut powers_to_test: Vec<u64> = prime_factors
        .iter()
        .map(|&factor| (p - 1) / factor)
        .collect();
    powers_to_test.dedup();

    let mut primitive_roots: Vec<u32> = Vec::new();

    for candidate in 2..(((p - 1) / 2) as u64) {
        if powers_to_test
            .iter()
            .all(|&power| candidate.pow(power as u32) % p != 1)
        {
            primitive_roots.push(candidate as u32);
        }
    }

    primitive_roots
}

fn find_prime_factors(p_minus: u64) -> Vec<u64> {
    let mut number = p_minus;
    let mut prime_numbers = Vec::<u64>::new();
    let mut candidate = 2;

    while number > 1 {
        while number % candidate == 0 {
            prime_numbers.push(candidate);
            number /= candidate;
        }
        candidate += 1;
    }

    prime_numbers
}

fn generate_p() -> u64 {
    let rand_0_1 = rand::random::<f32>();
    let floor = (rand_0_1 * 24.0 + 6.0) as u32;

    let mut pset = PrimeSet::new();
    let (_, p) = pset.find(floor as u64);
    p
}
