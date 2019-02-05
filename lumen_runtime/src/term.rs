#![cfg_attr(not(test), allow(dead_code))]

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

struct AtomIndex(pub usize);

struct AtomTable {
    names: Vec<String>,
}

impl AtomTable {
    fn find_or_insert(&mut self, name: &str) -> AtomIndex {
        let found_or_new_index = match self
            .names
            .iter()
            .position(|existing_name| existing_name == name)
        {
            Some(index) => index,
            None => {
                self.names.push(name.to_string());
                self.names.len() - 1
            }
        };

        AtomIndex(found_or_new_index)
    }

    fn new() -> AtomTable {
        AtomTable { names: Vec::new() }
    }
}

struct Env {
    atom_table: AtomTable,
}

#[derive(Debug, PartialEq)]
// MUST be `repr(u*)` so that size and layout is fixed for direct LLVM IR checking of tags
#[repr(usize)]
enum Tag {
    Arity = 0b0000_00,
    BinaryAggregate = 0b0001_00,
    PositiveBigNumber = 0b0010_00,
    NegativeBigNumber = 0b0011_00,
    Reference = 0b0100_00,
    Function = 0b0101_00,
    Float = 0b0110_00,
    Export = 0b0111_00,
    ReferenceCountedBinary = 0b1000_00,
    HeapBinary = 0b1001_00,
    Subbinary = 0b1010_00,
    ExternalPid = 0b1100_00,
    ExternalPort = 0b1101_00,
    ExternalReference = 0b1110_00,
    Map = 0b1111_00,
    List = 0b01,
    Boxed = 0b10,
    LocalPid = 0b00_11,
    LocalPort = 0b01_11,
    Atom = 0b00_10_11,
    CatchPointer = 0b01_10_11,
    EmptyList = 0b11_10_11,
    SmallInteger = 0b11_11,
}

struct TagError {
    tag: usize,
    bit_count: usize,
}

impl Display for TagError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{tag:0bit_count$b} is not a valid Term tag",
            tag = self.tag,
            bit_count = self.bit_count
        )
    }
}

const PRIMARY_TAG_MASK: usize = 0b11;
const HEADER_PRIMARY_TAG: usize = 0b00;
const HEADER_PRIMARY_TAG_MASK: usize = 0b1111_11;
const IMMEDIATE_PRIMARY_TAG_MASK: usize = 0b11_11;
const IMMEDIATE_IMMEDIATE_PRIMARY_TAG_MASK: usize = 0b11_11_11;

impl TryFrom<usize> for Tag {
    type Error = TagError;

    fn try_from(bits: usize) -> Result<Self, Self::Error> {
        match bits & PRIMARY_TAG_MASK {
            HEADER_PRIMARY_TAG => match bits & HEADER_PRIMARY_TAG_MASK {
                0b0000_00 => Ok(Tag::Arity),
                0b0001_00 => Ok(Tag::BinaryAggregate),
                0b0010_00 => Ok(Tag::PositiveBigNumber),
                0b0011_00 => Ok(Tag::NegativeBigNumber),
                0b0100_00 => Ok(Tag::Reference),
                0b0101_00 => Ok(Tag::Function),
                0b0110_00 => Ok(Tag::Float),
                0b0111_00 => Ok(Tag::Export),
                0b1000_00 => Ok(Tag::ReferenceCountedBinary),
                0b1001_00 => Ok(Tag::HeapBinary),
                0b1010_00 => Ok(Tag::Subbinary),
                0b1100_00 => Ok(Tag::ExternalPid),
                0b1101_00 => Ok(Tag::ExternalPort),
                0b1110_00 => Ok(Tag::ExternalReference),
                0b1111_00 => Ok(Tag::Map),
                tag => Err(TagError { tag, bit_count: 6 }),
            },
            0b01 => Ok(Tag::List),
            0b10 => Ok(Tag::Boxed),
            0b11 => match bits & IMMEDIATE_PRIMARY_TAG_MASK {
                0b00_11 => Ok(Tag::LocalPid),
                0b01_11 => Ok(Tag::LocalPort),
                0b10_11 => match bits & IMMEDIATE_IMMEDIATE_PRIMARY_TAG_MASK {
                    0b00_10_11 => Ok(Tag::Atom),
                    0b01_10_11 => Ok(Tag::CatchPointer),
                    0b11_10_11 => Ok(Tag::EmptyList),
                    tag => Err(TagError { tag, bit_count: 6 }),
                },
                0b11_11 => Ok(Tag::SmallInteger),
                tag => Err(TagError { tag, bit_count: 4 }),
            },
            tag => Err(TagError { tag, bit_count: 2 }),
        }
    }
}

#[derive(Debug)]
// MUST be `repr(C)` so that size and layout is fixed for direct LLVM IR checking of tags
#[repr(C)]
struct Term {
    tagged: usize,
}

impl Env {
    fn new() -> Env {
        Env {
            atom_table: AtomTable::new(),
        }
    }

    fn find_or_insert_atom(&mut self, name: &str) -> Result<Term, AtomIndexOverflow> {
        self.atom_table.find_or_insert(name).try_into()
    }
}

struct BadArgument;

impl Debug for BadArgument {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "bad argument")
    }
}

#[derive(Debug)]
enum LengthError {
    BadArgument(BadArgument),
    SmallIntegerOverflow(SmallIntegerOverflow),
}

impl Term {
    const EMPTY_LIST: Term = Term {
        tagged: Tag::EmptyList as usize,
    };

    fn tag(&self) -> Tag {
        match (self.tagged as usize).try_into() {
            Ok(tag) => tag,
            Err(tag_error) => panic!(tag_error),
        }
    }

    fn is_atom(&self, mut env: &mut Env) -> Result<Term, AtomIndexOverflow> {
        (self.tag() == Tag::Atom).try_into_term(&mut env)
    }

    fn is_empty_list(&self, mut env: &mut Env) -> Result<Term, AtomIndexOverflow> {
        (self.tag() == Tag::EmptyList).try_into_term(&mut env)
    }

    fn is_integer(&self, mut env: &mut Env) -> Result<Term, AtomIndexOverflow> {
        match self.tag() {
            Tag::SmallInteger => true,
            _ => false,
        }
        .try_into_term(&mut env)
    }

    fn length(&self, mut env: &mut Env) -> Result<Term, LengthError> {
        match self.tag() {
            Tag::EmptyList => 0.try_into_term(&mut env).map_err(|small_integer_overflow| {
                LengthError::SmallIntegerOverflow(small_integer_overflow)
            }),
            _ => Err(LengthError::BadArgument(BadArgument)),
        }
    }
}

/// Attempt to construct `Term` via a conversion.
trait TryIntoTerm {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion
    fn try_into_term(self, env: &mut Env) -> Result<Term, Self::Error>;
}

impl TryIntoTerm for bool {
    type Error = AtomIndexOverflow;

    fn try_into_term(self: Self, env: &mut Env) -> Result<Term, AtomIndexOverflow> {
        let name = if self { "true" } else { "false" };

        env.find_or_insert_atom(name)
    }
}

struct SmallIntegerOverflow {
    value: isize,
}

const SMALL_INTEGER_TAG_BIT_COUNT: u8 = 4;
const MIN_SMALL_INTEGER: isize = std::isize::MIN >> SMALL_INTEGER_TAG_BIT_COUNT;
const MAX_SMALL_INTEGER: isize = std::isize::MAX >> SMALL_INTEGER_TAG_BIT_COUNT;

impl Debug for SmallIntegerOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Integer ({}) does not fit in small integer range ({}..{})",
            self.value, MIN_SMALL_INTEGER, MAX_SMALL_INTEGER
        )
    }
}

impl TryIntoTerm for isize {
    type Error = SmallIntegerOverflow;

    fn try_into_term(self: Self, _env: &mut Env) -> Result<Term, SmallIntegerOverflow> {
        if MIN_SMALL_INTEGER <= self && self <= MAX_SMALL_INTEGER {
            Ok(Term {
                tagged: ((self as usize) << SMALL_INTEGER_TAG_BIT_COUNT)
                    | (Tag::SmallInteger as usize),
            })
        } else {
            Err(SmallIntegerOverflow { value: self })
        }
    }
}

struct AtomIndexOverflow {
    atom_index: AtomIndex,
}

impl AtomIndexOverflow {
    fn new(atom_index: AtomIndex) -> AtomIndexOverflow {
        AtomIndexOverflow { atom_index }
    }
}

const ATOM_TAG_BIT_COUNT: u8 = 6;
const MAX_ATOM_INDEX: usize = (std::usize::MAX << ATOM_TAG_BIT_COUNT) >> ATOM_TAG_BIT_COUNT;

impl Debug for AtomIndexOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "index ({}) in atom table exceeds max index that can be tagged as an atom in a Term ({})",
            self.atom_index.0,
            MAX_ATOM_INDEX
        )
    }
}

impl TryFrom<AtomIndex> for Term {
    type Error = AtomIndexOverflow;

    fn try_from(atom_index: AtomIndex) -> Result<Self, AtomIndexOverflow> {
        if atom_index.0 <= MAX_ATOM_INDEX {
            Ok(Term {
                tagged: (atom_index.0 << ATOM_TAG_BIT_COUNT) | (Tag::Atom as usize),
            })
        } else {
            Err(AtomIndexOverflow::new(atom_index))
        }
    }
}

/// All terms in Erlang and Elixir are completely ordered.
///
/// number < atom < reference < function < port < pid < tuple < map < list < bitstring
///
/// > When comparing two numbers of different types (a number being either an integer or a float), a
/// > conversion to the type with greater precision will always occur, unless the comparison
/// > operator used is either === or !==. A float will be considered more precise than an integer,
/// > unless the float is greater/less than +/-9007199254740992.0 respectively, at which point all
/// > the significant figures of the float are to the left of the decimal point. This behavior
/// > exists so that the comparison of large numbers remains transitive.
/// >
/// > The collection types are compared using the following rules:
/// >
/// > * Tuples are compared by size, then element by element.
/// > * Maps are compared by size, then by keys in ascending term order, then by values in key
/// order. >   In the specific case of maps' key ordering, integers are always considered to be less
/// than >   floats.
/// > * Lists are compared element by element.
/// > * Bitstrings are compared byte by byte, incomplete bytes are compared bit by bit.
/// > -- https://hexdocs.pm/elixir/operators.html#term-ordering
impl std::cmp::PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        let tag = self.tag();

        if tag == other.tag() {
            match tag {
                Tag::Atom | Tag::SmallInteger => self.tagged == other.tagged,
                _ => unimplemented!(),
            }
        } else {
            false
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl std::cmp::Eq for Term {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoms_with_same_name_have_same_tagged_value() {
        let mut env = Env::new();
        assert_eq!(
            env.find_or_insert_atom("atom").unwrap().tagged,
            env.find_or_insert_atom("atom").unwrap().tagged
        )
    }

    #[test]
    fn atoms_have_atom_tag() {
        let mut env = Env::new();
        assert_eq!(env.find_or_insert_atom("true").unwrap().tag(), Tag::Atom);
        assert_eq!(env.find_or_insert_atom("false").unwrap().tag(), Tag::Atom);
    }

    #[test]
    fn booleans_are_atoms() {
        let mut env = Env::new();
        let true_term = env.find_or_insert_atom("true").unwrap();
        let false_term = env.find_or_insert_atom("false").unwrap();

        assert_eq!(true_term.is_atom(&mut env).unwrap(), true_term);
        assert_eq!(false_term.is_atom(&mut env).unwrap(), true_term);
    }

    #[test]
    fn nil_is_atom() {
        let mut env = Env::new();
        let nil_term = env.find_or_insert_atom("nil").unwrap();
        let true_term = env.find_or_insert_atom("true").unwrap();

        assert_eq!(nil_term.is_atom(&mut env).unwrap(), true_term);
    }

    #[test]
    fn atom_is_not_empty_list() {
        let mut env = Env::new();
        let nil_term = env.find_or_insert_atom("nil").unwrap();
        let false_term = false.try_into_term(&mut env).unwrap();

        assert_eq!(nil_term.is_empty_list(&mut env).unwrap(), false_term);
    }

    #[test]
    fn empty_list_is_empty_list() {
        let mut env = Env::new();

        assert_eq!(
            Term::EMPTY_LIST.is_empty_list(&mut env).unwrap(),
            env.find_or_insert_atom("true").unwrap()
        );
    }

    #[test]
    fn small_integer_is_integer() {
        let mut env = Env::new();
        let zero_term = 0.try_into_term(&mut env).unwrap();
        let true_term = true.try_into_term(&mut env).unwrap();

        assert_eq!(zero_term.is_integer(&mut env).unwrap(), true_term);
    }

    #[test]
    fn empty_list_length_is_zero() {
        let mut env = Env::new();
        let zero_term = 0.try_into_term(&mut env).unwrap();

        assert_eq!(Term::EMPTY_LIST.length(&mut env).unwrap(), zero_term);
    }
}
