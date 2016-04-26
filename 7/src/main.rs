use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug)]
enum Input {
   Value(u16),
   Element(String),
   None,
}

#[derive(Debug)]
struct InvalidInput;

impl FromStr for Input {
   type Err = InvalidInput;
   fn from_str(s: &str) -> Result<Input, InvalidInput> {
      return match s.parse::<u16>() {
         Ok(v) => Ok(Input::Value(v)),
         Err(_) => Ok(Input::Element(s.to_string())),
      };
   }
}

#[derive(Debug)]
enum Operation {
   Value,
   Not,
   And,
   Or,
   LShift,
   RShift,
}

impl FromStr for Operation {
   type Err = InvalidInput;
   fn from_str(s: &str) -> Result<Operation, InvalidInput> {
      return match s {
         "OR" => Ok(Operation::Or),
         "AND" => Ok(Operation::And),
         "LSHIFT" => Ok(Operation::LShift),
         "RSHIFT" => Ok(Operation::RShift),
         _ => Err(InvalidInput)
      }
   }
}


#[derive(Debug)]
struct ElementSpec {
   left: Input,
   right: Input,
   op: Operation,
}

impl FromStr for ElementSpec {
   type Err = InvalidInput;
   fn from_str(s: &str) -> Result<ElementSpec, InvalidInput> {
      let parts = s.split(" ").collect::<Vec<_>>();

      // Either passthru or Value
      if parts.len() == 1 {
         return Ok(ElementSpec{
            left: parts[0].parse::<Input>().unwrap(),
            right: Input::None,
            op: Operation::Value,
         });
      } else if parts.len() == 2 {
         return Ok(ElementSpec {
            left: parts[1].parse::<Input>().unwrap(),
            right: Input::None,
            op: Operation::Not,
         });
      } else if parts.len() == 3 {
         return Ok(ElementSpec {
            left:  parts[0].parse::<Input>().unwrap(),
            right:  parts[2].parse::<Input>().unwrap(),
            op: parts[1].parse::<Operation>().unwrap(),
         });
      } else {
         return Err(InvalidInput)
      }
   }
}

impl ElementSpec {
   fn evaluate(&self, left: u16, right: u16) -> u16 {
      match self.op {
         Operation::Value => left,
         Operation::Not => !left,
         Operation::And => left & right,
         Operation::Or => left | right,
         Operation::LShift => left << right,
         Operation::RShift => left >> right,
      }
   }
}

#[derive(Debug)]
struct Element {
   spec: ElementSpec,
   name: String,
   value: Option<u16>,
}

impl Element {
   fn set_value(&mut self, val: u16) {
      self.value = Some(val);
      println!("setting {} as {}", self.name, val);
   }

   fn clear_value(&mut self) {
      self.value = None;
      println!("Clearing {}", self.name);
   }
}

#[derive(Debug)]
struct Circuit {
   parts: HashMap<String, Element>,
}

impl Circuit {
   fn new() -> Circuit {
      Circuit{parts: HashMap::new()}
   }

   fn add_element(&mut self, name: &str, spec: &str) {
      self.parts.insert(name.to_string(),
                   Element {
                     spec: spec.parse::<ElementSpec>().unwrap(),
                     name: name.to_string(),
                     value: None,
                   });
   }

   fn resolve_input(&mut self, input: &Input) -> u16 {
      match input {
         &Input::None => 0,
         &Input::Value(ref v) => *v,
         &Input::Element(ref e) => self.get_value(&e),
      }
   }

   fn get_value(&mut self, name: &str) -> u16 {
       let do_update = match self.parts.get(name) {
           Some(&Element { value: Some(v), .. }) => return v,
           Some(_) => true,
           None => false,
       };
       if !do_update { return 0; }
       let mut to_update = self.parts.remove(name).expect("where'd it go");
       let ret = to_update.spec.evaluate(
           self.resolve_input(&to_update.spec.left),
           self.resolve_input(&to_update.spec.right));
       to_update.set_value(ret);
       if let Some(prev) = self.parts.insert(name.to_string(), to_update) {
           panic!("circular reference? something re-inserted {:?} under us: {:?}", name, prev);
       }
       return ret;
   }
   fn clear_cache(&mut self) {
      for (_, v) in &mut self.parts {
         v.clear_value();
      }
   }
   fn force_value(&mut self, name: &str, val: u16) {
      if let Some(e) = self.parts.get_mut(&name.to_string()) {
         e.set_value(val);
      }
   }

}

fn main() {
   let f = File::open("input.txt").unwrap();
   let line_buffer = BufReader::new(&f);

   let mut circuit = Circuit::new();
   for line in line_buffer.lines() {
      let curr = line.unwrap();
      let parts = curr.split(" -> ").collect::<Vec<_>>();
      circuit.add_element(parts[1], parts[0]);
   }

   let a = circuit.get_value("a");
   circuit.clear_cache();
   circuit.force_value("b", a);
   println!("a is {}", circuit.get_value("a"));

}

#[test]
fn test_number() {
   let mut circuit = Circuit::new();
   circuit.add_element("x", "123");
   circuit.add_element("y", "456");
   circuit.add_element("d", "x AND y");
   circuit.add_element("e", "x OR y");
   circuit.add_element("f", "x LSHIFT 2");
   circuit.add_element("g", "y RSHIFT 2");
   circuit.add_element("h", "NOT x");
   circuit.add_element("i", "NOT y");

   assert_eq!(circuit.get_value("d"), 72);
   assert_eq!(circuit.get_value("e"), 507);
   assert_eq!(circuit.get_value("f"), 492);
   assert_eq!(circuit.get_value("g"), 114);
   assert_eq!(circuit.get_value("h"), 65412);
   assert_eq!(circuit.get_value("i"), 65079);
   assert_eq!(circuit.get_value("x"), 123);
   assert_eq!(circuit.get_value("y"), 456);
}
