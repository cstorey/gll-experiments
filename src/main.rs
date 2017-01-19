// Inspired by https://github.com/tomjridge/ocaml_gll_examples/blob/master/gll.ml
use std::collections::VecDeque;

#[derive(Clone,Copy,Debug, Eq,PartialEq,Hash)]
enum Label {
    LS,
    L0,
    LS1,
    L1,
    L2,
    LS2,
    L3,
    L4,
    LS3,
    LA,
    LB,
}

#[derive(Clone,Copy,Debug, Eq,PartialEq,Hash)]
struct Extlbl(usize, Label);

impl Extlbl {
    fn to_label(&self) -> Label {
        self.1
    }
}

type Stack = VecDeque<Extlbl>;

#[derive(Clone,Debug, Eq,PartialEq,Hash)]
struct Descriptor(Label, Stack, usize);

type R = VecDeque<Descriptor>;

#[derive(Debug, Eq,PartialEq)]
struct State {
    pc: Label,
    idx: usize,
    r: R,
    stack: Stack,
    input: Vec<char>,
}

#[derive(Clone,Debug, Eq,PartialEq,Hash)]
enum Status {
    Success,
    Failure,
    Continue,
}

impl State {
    fn pop(&mut self, i: usize) {
        self.pc = Label::L0;
        let mut stack = self.stack.clone();
        let head = stack.pop_back().expect("stack top");
        let descr = Descriptor(head.to_label(), stack, i);
        println!("Pop: {:?}", descr);
        self.r.push_back(descr);
    }

    fn push(&mut self, lbl: Label, idx: usize) {
        println!("Push: {:?}; {:?}", lbl, idx);
        self.stack.push_back(Extlbl(idx, lbl))
    }

    fn step(&mut self) -> Status {
        match self.pc {
            Label::LS => {
                let inp = self.input.get(self.idx).cloned();
                if [Some('a'), Some('c')].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS1, self.stack.clone(), self.idx));
                }
                if [Some('a'), Some('b')].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS2, self.stack.clone(), self.idx));
                }
                if [Some('d'), None].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS3, self.stack.clone(), self.idx));
                }

                self.pc = Label::L0;
            }
            Label::L0 => {
                if let Some(next) = self.r.pop_front() {
                    println!("Next: {:?}", next);
                    let Descriptor(lbl, st, i) = next;
                    println!("L0? {:?}; stack empty? {:?}; {:?} == {:?}? {:?}",
                             lbl == Label::L0,
                             st.is_empty(),
                             i,
                             self.input.len(),
                             i == self.input.len());
                    if lbl == Label::L0 && st.is_empty() && i == self.input.len() {
                        return Status::Success;
                    } else {
                        self.pc = lbl;
                        self.stack = st;
                        self.idx = i;
                    }
                } else {
                    return Status::Failure;
                }
            }
            Label::LS1 => {
                let i = self.idx;
                self.push(Label::L1, i);
                self.pc = Label::LA;
            }
            Label::L1 => {
                let i = self.idx;
                self.pc = Label::LS;
                self.push(Label::L2, i);
            }
            Label::L2 => {
                let i = self.idx;
                if self.input.get(i) == Some(&'d') {
                    self.pop(i + 1)
                } else {
                    self.pc = Label::L0
                }
            }
            Label::LS2 => {
                let i = self.idx;
                self.push(Label::L3, i);
                self.pc = Label::LB;
            }
            Label::L3 => {
                let i = self.idx;
                self.push(Label::L4, i);
                self.pc = Label::LS;
            }
            Label::L4 => {
                let i = self.idx;
                self.pop(i)
            }
            Label::LS3 => {
                let i = self.idx;
                self.pop(i)
            }
            Label::LA => {
                let i = self.idx;
                let inp = self.input.get(i).cloned();
                if [Some('a'), Some('c')].contains(&inp) {
                    self.pop(i + 1)
                } else {
                    self.pc = Label::L0
                }
            }
            Label::LB => {
                let i = self.idx;
                let inp = self.input.get(i).cloned();
                if [Some('a'), Some('b')].contains(&inp) {
                    self.pop(i + 1)
                } else {
                    self.pc = Label::L0
                }
            }
            // other => panic!("Unimplemented: {:?}", other),
        }
        Status::Continue
    }
}

fn recognises(input: &str) -> bool {
    let mut s = State {
        pc: Label::LS,
        idx: 0,
        r: VecDeque::new(),
        stack: vec![Extlbl(0, Label::L0)].into_iter().collect(),
        input: input.chars().collect(),
    };

    loop {
        println!("{:?}", s);
        match s.step() {
            Status::Success => return true,
            Status::Failure => return false,
            Status::Continue => (),
        }
    }
}

fn main() {
    let input = "aad";
    let res = recognises(input);
    println!("{:?} => {:?}", input, res);
}
#[test]
fn recognises_empty_string() {
    assert!(recognises(""));
}
#[test]
fn recognises_aad() {
    assert!(recognises("aad"));
}

#[test]
fn recognises_b() {
    assert!(recognises("b"));
}

#[test]
fn rejects_z() {
    assert!(!recognises("z"));
}
#[test]
fn rejects_aadz() {
    assert!(!recognises("aadz"));
}
