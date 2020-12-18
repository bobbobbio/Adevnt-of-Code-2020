use bit_set::BitSet;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead};
use std::str::FromStr;
use std::{fmt, num};

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

fn parse_lines<R: BufRead, T: FromStr>(lines: R) -> Result<Vec<T>, Error>
where
    Error: From<<T as FromStr>::Err>,
{
    let mut values = vec![];
    for maybe_line in lines.lines() {
        values.push(maybe_line?.parse()?);
    }
    Ok(values)
}

#[derive(Debug, Copy, Clone)]
struct AdapterMismatch;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Adapter {
    rating: u64,
}

impl fmt::Debug for Adapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.rating)
    }
}

impl Adapter {
    fn new(rating: u64) -> Self {
        Self { rating }
    }

    fn can_plug_into(&self, other: &Self) -> bool {
        let rating_difference = self.rating as i128 - other.rating as i128;
        rating_difference >= 1 && rating_difference <= 3
    }

    fn joltage_jump(&self, other: &Self) -> u64 {
        other.rating - self.rating
    }
}

impl fmt::Debug for AdapterChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.adapters[0])?;
        for a in &self.adapters[1..] {
            write!(f, "-> {:?}", a)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct AdapterChain {
    adapters: Vec<Adapter>,
}

impl AdapterChain {
    fn new() -> Self {
        Self::with_adapter(Adapter::new(0))
    }

    fn with_adapter(adapter: Adapter) -> Self {
        Self {
            adapters: vec![adapter],
        }
    }

    fn end_adapter(&self) -> &Adapter {
        self.adapters.last().unwrap()
    }

    fn plug(&mut self, adapter: Adapter) {
        assert!(adapter.can_plug_into(self.end_adapter()));
        self.adapters.push(adapter);
    }

    fn try_plug(&mut self, adapter: Adapter) -> Result<(), AdapterMismatch> {
        if adapter.can_plug_into(self.end_adapter()) {
            self.adapters.push(adapter);
            Ok(())
        } else {
            Err(AdapterMismatch)
        }
    }

    fn unplug(&mut self) {
        self.adapters.remove(self.adapters.len() - 1);
    }

    fn count_joltage_jumps(&self, jump_to_count: u64) -> usize {
        let adapter_pairs = self.adapters.iter().zip(self.adapters[1..].iter());
        let joltage_jumps = adapter_pairs.map(|(a, b)| a.joltage_jump(b));
        joltage_jumps.filter(|&j| j == jump_to_count).count()
    }
}

struct AdapterCompatMap(HashMap<Adapter, Vec<Adapter>>);

impl AdapterCompatMap {
    fn new(adapters: &[Adapter]) -> Self {
        let mut map = HashMap::new();
        for (i, a) in adapters.iter().enumerate() {
            let mut compatible = vec![];
            for (j, b) in adapters.iter().enumerate() {
                if i == j {
                    continue;
                }
                if b.can_plug_into(a) {
                    compatible.push(b.clone());
                }
            }
            compatible.sort();
            map.insert(a.clone(), compatible);
        }
        Self(map)
    }

    fn get_compatible_for(&self, adapter: &Adapter) -> &[Adapter] {
        &self.0.get(adapter).unwrap()
    }
}

#[derive(Hash, Clone, PartialEq, Eq)]
struct AdapterCollection(BitSet);

impl AdapterCollection {
    fn new(adapters: Vec<Adapter>) -> Self {
        let mut set = BitSet::new();
        for a in adapters {
            set.insert(a.rating as usize);
        }
        Self(set)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn try_remove(&mut self, adapter: &Adapter) -> Option<Adapter> {
        if self.0.remove(adapter.rating as usize) {
            Some(adapter.clone())
        } else {
            None
        }
    }

    fn add_adapter(&mut self, adapter: Adapter) {
        self.0.insert(adapter.rating as usize);
    }
}

fn find_adapter_chain_inner(
    compat_map: &AdapterCompatMap,
    chain: &mut AdapterChain,
    device_adapter: Adapter,
    adapters: &mut AdapterCollection,
) -> Result<(), AdapterMismatch> {
    if adapters.is_empty() {
        return chain.try_plug(device_adapter);
    }

    let compatible = compat_map.get_compatible_for(chain.end_adapter());

    for c in compatible {
        if let Some(new_end) = adapters.try_remove(c) {
            chain.plug(new_end.clone());

            let res = find_adapter_chain_inner(compat_map, chain, device_adapter.clone(), adapters);
            if res.is_ok() {
                return Ok(());
            }
            chain.unplug();
            adapters.add_adapter(new_end);
        }
    }
    Err(AdapterMismatch)
}

fn find_adapter_chain(device_adapter: Adapter, adapters: Vec<Adapter>) -> AdapterChain {
    let mut all_adapters = adapters.clone();
    all_adapters.push(Adapter::new(0));
    all_adapters.push(device_adapter.clone());
    let compat_map = AdapterCompatMap::new(&all_adapters);

    let mut chain = AdapterChain::new();
    let mut adapters = AdapterCollection::new(adapters);
    find_adapter_chain_inner(&compat_map, &mut chain, device_adapter, &mut adapters).unwrap();
    chain
}

fn part_one(adapter_ratings: &[u64]) {
    let max_adapter_rating = adapter_ratings.iter().fold(0, |a, &b| a.max(b));
    let device_adapter = Adapter::new(max_adapter_rating + 3);

    let adapters: Vec<_> = adapter_ratings.iter().cloned().map(Adapter::new).collect();
    let chain = find_adapter_chain(device_adapter, adapters);
    let answer = chain.count_joltage_jumps(1) * chain.count_joltage_jumps(3);
    println!("{}", answer);
}

fn cache_key(end: &Adapter, adapters: &AdapterCollection) -> u64 {
    let mut s = DefaultHasher::new();
    end.hash(&mut s);
    for a in adapters.0.iter() {
        if a as u64 > end.rating {
            a.hash(&mut s);
        }
    }
    s.finish()
}

fn count_adapter_chains_inner(
    cache: &mut HashMap<u64, usize>,
    compat_map: &AdapterCompatMap,
    chain: &mut AdapterChain,
    device_adapter: Adapter,
    adapters: &mut AdapterCollection,
) -> usize {
    let key = cache_key(chain.end_adapter(), adapters);
    if let Some(v) = cache.get(&key) {
        return *v;
    }

    let mut chains = 0;
    if chain.try_plug(device_adapter.clone()).is_ok() {
        chains += 1;
        chain.unplug();
    }

    let compatible = compat_map.get_compatible_for(chain.end_adapter());

    for c in compatible {
        if let Some(new_end) = adapters.try_remove(c) {
            chain.plug(new_end.clone());

            let inner_chains = count_adapter_chains_inner(
                cache,
                compat_map,
                chain,
                device_adapter.clone(),
                adapters,
            );
            chains += inner_chains;
            chain.unplug();
            adapters.add_adapter(new_end);
        }
    }

    cache.insert(key, chains);
    chains
}

fn count_adapter_chains(device_adapter: Adapter, adapters: Vec<Adapter>) -> usize {
    let mut all_adapters = adapters.clone();
    all_adapters.push(Adapter::new(0));
    all_adapters.push(device_adapter.clone());
    let compat_map = AdapterCompatMap::new(&all_adapters);

    let mut cache = HashMap::new();
    let mut chain = AdapterChain::new();
    let mut adapters = AdapterCollection::new(adapters);
    let res = count_adapter_chains_inner(
        &mut cache,
        &compat_map,
        &mut chain,
        device_adapter,
        &mut adapters,
    );
    res
}

fn part_two(adapter_ratings: &[u64]) {
    let max_adapter_rating = adapter_ratings.iter().fold(0, |a, &b| a.max(b));
    let device_adapter = Adapter::new(max_adapter_rating + 3);

    let adapters: Vec<_> = adapter_ratings.iter().cloned().map(Adapter::new).collect();
    let answer = count_adapter_chains(device_adapter, adapters);
    println!("{}", answer);
}

fn main() -> Result<(), Error> {
    let adapter_ratings: Vec<u64> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&adapter_ratings);

    println!("Part 2");
    part_two(&adapter_ratings);

    Ok(())
}
