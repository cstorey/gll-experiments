// Inspired by https://github.com/tomjridge/ocaml_gll_examples/blob/master/gll.ml
use std::collections::{BTreeSet, BTreeMap, VecDeque};

/*
Grammar: Γ0
    S ::= ASd | BS | ε
    // GLL blocks terminated by "∙": A∙S∙d | B∙S∙ | ε∙
    A ::= a|c
    B ::= a|b
*/

#[derive(Clone,Copy,Debug, Eq,PartialEq,Hash,Ord,PartialOrd)]
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

#[derive(Clone,Copy,Debug, Eq,PartialEq,Hash,Ord,PartialOrd)]
enum Extlbl {
    Positioned(usize, Label),
    Dollar,
}

impl Extlbl {
    fn to_label(&self) -> Label {
        match self {
            &Extlbl::Positioned(_, lbl) => lbl,
            _ => panic!("... eh"),
        }
    }
}

type Stack = VecDeque<Extlbl>;

#[derive(Clone,Debug, Eq,PartialEq,Hash,Ord,PartialOrd)]
struct Descriptor(Label, Extlbl, usize);

#[derive(Debug, Eq,PartialEq)]
struct State {
    pc: Label,
    idx: usize,
    r: VecDeque<Descriptor>,
    // In the paper this is expressed as U_i, or what seems to be a i -> Set<(L, u)>
    // where `u` is a GSS node.
    seen: BTreeMap<usize, BTreeSet<(Label, Extlbl)>>,
    // May need to be Extlbl -> Set Extlbl
    gss: BTreeMap<Extlbl, BTreeSet<Extlbl>>,
    // `P`.
    popped: BTreeSet<(Extlbl, usize)>,
    // Current GSS node
    u: Extlbl,
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
        /*
        self.pc = Label::L0;
        let mut stack = self.stack.clone();
        let head = stack.pop_back().expect("stack top");
        let descr = Descriptor(head.to_label(), stack, i);
        println!("Pop: {:?}", descr);
        self.r.push_back(descr);
        */

        let lbl = self.u.to_label();
        self.popped.insert((self.u, i));
        let children =
            self.gss.get(&self.u).into_iter().flat_map(|vs| vs).cloned().collect::<Vec<_>>();
        for v in children {
            self.add(lbl, v, i)
        }
    }

    fn add(&mut self, lbl: Label, u: Extlbl, idx: usize) {
        let ui = self.seen.entry(idx).or_insert_with(BTreeSet::new);
        ui.insert((lbl, u));
        self.r.push_back(Descriptor(lbl, u, idx));
    }

    fn create(&mut self, lbl: Label, idx: usize) {
        println!("create: {:?}; {:?}", lbl, idx);
        let v = Extlbl::Positioned(idx, lbl);
        let mut toadd = Vec::new();
        {
            let mut fromv = self.gss.entry(v).or_insert_with(BTreeSet::new);
            if !fromv.contains(&self.u) {
                fromv.insert(self.u);
                for &(_, k) in self.popped.iter().filter(|&&(ref vp, _)| vp == &v) {
                    toadd.push((lbl, self.u, k))
                }
            }
        }

        for (lbl, u, k) in toadd {
            self.add(lbl, u, k)
        }

        self.u = v;
    }

    fn step(&mut self) -> Status {
        match self.pc {
            Label::LS => {
                let inp = self.input.get(self.idx).cloned();
                if [Some('a'), Some('c')].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS1, self.u.clone(), self.idx));
                }
                if [Some('a'), Some('b')].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS2, self.u.clone(), self.idx));
                }
                if [Some('d'), None].contains(&inp) {
                    self.r.push_back(Descriptor(Label::LS3, self.u.clone(), self.idx));
                }

                self.pc = Label::L0;
            }
            Label::L0 => {
                if let Some(next) = self.r.pop_front() {
                    println!("Next: {:?}", next);
                    let Descriptor(lbl, u, i) = next;
                    self.pc = lbl;
                    self.u = u;
                    self.idx = i;
                } else {
                    let u0 = Extlbl::Dollar;
                    if self.seen
                        .entry(self.input.len())
                        .or_insert_with(BTreeSet::new)
                        .contains(&(Label::L0, u0)) {
                        return Status::Success;
                    } else {
                        return Status::Failure;
                    }
                }
            }
            Label::LS1 => {
                let i = self.idx;
                self.create(Label::L1, i);
                self.pc = Label::LA;
            }
            Label::L1 => {
                let i = self.idx;
                self.create(Label::L2, i);
                self.pc = Label::LS;
            }
            Label::L2 => {
                let i = self.idx;
                if self.input.get(i) == Some(&'d') {
                    self.pop(i + 1)
                }
                self.pc = Label::L0
            }
            Label::LS2 => {
                let i = self.idx;
                self.create(Label::L3, i);
                self.pc = Label::LB;
            }
            Label::L3 => {
                let i = self.idx;
                self.create(Label::L4, i);
                self.pc = Label::LS;
            }
            Label::L4 => {
                let i = self.idx;
                self.pop(i);
                self.pc = Label::L0;

            }
            Label::LS3 => {
                let i = self.idx;
                self.pop(i);
                self.pc = Label::L0;
            }
            Label::LA => {
                let i = self.idx;
                let inp = self.input.get(i).cloned();
                if [Some('a'), Some('c')].contains(&inp) {
                    self.pop(i + 1)
                }
                self.pc = Label::L0
            }
            Label::LB => {
                let i = self.idx;
                let inp = self.input.get(i).cloned();
                if [Some('a'), Some('b')].contains(&inp) {
                    self.pop(i + 1)
                }
                self.pc = Label::L0
            }
            // other => panic!("Unimplemented: {:?}", other),
        }
        Status::Continue
    }
}

fn recognises(input: &str) -> bool {
    let u0 = Extlbl::Dollar;
    let u1 = Extlbl::Positioned(0, Label::L0);
    let mut s = State {
        pc: Label::LS,
        idx: 0,
        u: u1,
        r: VecDeque::new(),
        gss: BTreeMap::new(),
        popped: BTreeSet::new(),
        seen: BTreeMap::new(),
        input: input.chars().collect(),
    };
    // stack: vec![Extlbl::Positioned(0, Label::L0)].into_iter().collect(),
    s.gss.entry(u1).or_insert_with(BTreeSet::new).insert(u0);

    loop {
        println!("{:#?}", s);
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
//#[ignore]
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
