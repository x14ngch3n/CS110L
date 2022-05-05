/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

#[allow(unused)]
fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    v.iter().map(|x| x + n).collect()
}

#[allow(unused)]
fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for num in v.iter_mut() {
        *num += n;
    }
}

#[allow(unused)]
fn dedup(v: &mut Vec<i32>) {
    // collect v into hashset
    let mut hs = HashSet::<i32>::new();
    // Don't directly convert hashset back to vector because it does not gurantee order.
    // find if v's element in hs
    let mut index = 0;
    while index < v.len() {
        match hs.contains(&v[index]) {
            true => {
                v.remove(index);
            }
            false => {
                hs.insert(v[index]);
                index += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
