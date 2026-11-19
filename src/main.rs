use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;

macro_rules! nfa {
    ($($($state:expr),*)=>*) => {{
        // start nfa with initial epsion state

        let mut states = vec![];
        $(
            // for each => create and epsilon glue
            let epsilon_index = states.len() as u64;
            let mut next_state_raw_ptrs = vec![];

            $(
                states.push((states.len() as u64 + 1, State::new(Some($state), vec![0], false)));
                let len = states.len() - 1;
                next_state_raw_ptrs.push(&raw mut states[len].1.next_states[0]);
            )*

            states.push((epsilon_index, State::new(None, ((epsilon_index + 1)..=states.len() as u64).into_iter().collect(), false)));

            for raw_ptr in next_state_raw_ptrs.into_iter() {
                unsafe { *raw_ptr = states.len() as u64 };
            }

            if epsilon_index != 0 {
                states.push((states.len() as u64, State::new(None, vec![(states.len() + 1) as u64, epsilon_index], false)));
            } else {
                states.push((states.len() as u64, State::new(None, vec![(states.len() + 1) as u64], false)));
            }
        )*

        states.push((states.len() as u64, State::new(None, vec![], true)));

        NFA::new(states.into_iter().collect())
    }};
}

#[derive(Debug)]
struct State<T: PartialEq<D> + Debug, D: Debug> {
    alphabet: Option<T>,
    next_states: Vec<u64>,
    accepting: bool,
    phantom_data: PhantomData<D>,
}

#[derive(PartialEq, Debug)]
enum Regex {
    Letter,
    Digit,
    Char(char),
}

impl PartialEq<char> for Regex {
    fn eq(&self, other: &char) -> bool {
        match self {
            Self::Letter => other.is_ascii_alphabetic(),
            Self::Digit => other.is_ascii_digit(),
            Self::Char(c) => c == other,
        }
    }
}

impl<T: PartialEq<D> + Debug, D: Debug> State<T, D> {
    pub fn new(alphabet: Option<T>, next_states: Vec<u64>, accepting: bool) -> Self {
        Self {
            alphabet,
            next_states,
            accepting,
            phantom_data: PhantomData::default(),
        }
    }
}

#[derive(Debug)]
struct NFA<T: PartialEq<D> + Debug, D: Debug> {
    states: BTreeMap<u64, State<T, D>>,
}

impl<T: PartialEq<D> + Debug, D: Debug> NFA<T, D> {
    pub fn new(states: BTreeMap<u64, State<T, D>>) -> Self {
        Self { states }
    }
    pub fn trans<'a>(&'a self, state: &'a State<T, D>, input: &D) -> Option<&'a State<T, D>> {
        let mut states: Vec<&State<T, D>> = vec![state];

        while let Some(state) = states.pop() {
            for s in state.next_states.iter() {
                if let Some(state) = self.get_state_by_id(*s) {
                    if let Some(ref alphabet) = state.alphabet {
                        if alphabet == input {
                            return Some(state);
                        }
                    } else {
                        states.extend(self.eclosure(state));
                    }
                }
            }

            if state.accepting && states.is_empty() {
                return Some(state);
            }
        }

        None
    }
    pub fn eclosure(&self, state: &State<T, D>) -> Vec<&State<T, D>> {
        let mut e_states = vec![state];
        let mut ret_states = vec![];

        while let Some(e_state) = e_states.pop() {
            for s in e_state.next_states.iter() {
                if let Some(state) = self.get_state_by_id(*s) {
                    // if epsilon
                    if state.alphabet.is_none() {
                        e_states.push(state);
                        ret_states.push(state);
                    }
                }
            }
        }

        ret_states
    }
    pub fn start_state(&self) -> &State<T, D> {
        let (_, value) = self.states.first_key_value().unwrap();
        value
    }
    pub fn get_state_by_id(&self, id: u64) -> Option<&State<T, D>> {
        self.states.get(&id)
    }
}

fn main() {
    let string = String::from("small weak and petite hikki");

    let nfa: NFA<_, char> = nfa!(Regex::Letter => Regex::Letter , Regex::Digit , Regex::Char(' '));

    let mut states = vec![nfa.start_state()];

    let mut chars = string.chars().peekable();

    let mut new_str = String::new();

    while let Some(state) = states.pop() {
        if let Some(c) = chars.peek() {
            if let Some(state) = nfa.trans(state, c) {
                if state.accepting {
                    println!("Reached end of nfa computed str: {new_str}");
                    break;
                }
                // if we found state that takes are input consume input and go to next state
                println!("input consumed => {}", c);
                new_str.push(chars.next().unwrap());
                states.push(state);
            } else {
                println!("Encounterd syntax error in nfa invalid char => {c:?}");
            }
        }
    }
}

// RE -> NFA -> DFA
//(a)? (b*)?
// thompsons construction
