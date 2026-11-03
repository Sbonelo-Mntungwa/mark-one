//! A circuit as data: parts, and wires joining their pins. This is what the
//! editor draws, what gets saved, and what the simulator builds from.

use serde::{Deserialize, Serialize};

fn is_zero(v: &u8) -> bool {
    *v == 0
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PartKind {
    Vcc,
    Gnd,
    Nmos,
    Pmos,
    In,
    Out,
    Chip,
    /// 16-byte program memory with editable contents.
    Rom,
    /// 8-bit decimal readout. A sink: it only reads its pins.
    Display,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    pub kind: PartKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<usize>,
    /// ROM contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    /// Assembly source for a ROM, kept for the editor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    /// Quarter turns clockwise on the board. Drawing only: it never changes
    /// what the circuit does.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rot: u8,
    /// Pin name for an input or output part in free build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct End {
    pub part: String,
    pub pin: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wire {
    pub from: End,
    pub to: End,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Circuit {
    #[serde(default)]
    pub parts: Vec<Part>,
    #[serde(default)]
    pub wires: Vec<Wire>,
}

impl Circuit {
    /// Identity of the wiring, ignoring where parts sit on the board.
    pub fn topology_key(&self) -> String {
        let parts: Vec<_> = self
            .parts
            .iter()
            .map(|p| (&p.id, p.kind, &p.chip, p.pin, &p.data))
            .collect();
        let wires: Vec<_> = self
            .wires
            .iter()
            .map(|w| (&w.from.part, w.from.pin, &w.to.part, w.to.pin))
            .collect();
        serde_json::to_string(&(parts, wires)).unwrap_or_default()
    }
}

impl Part {
    /// A part with no chip type, pin, contents or label, at the origin.
    pub fn new(id: &str, kind: PartKind) -> Part {
        Part {
            id: id.into(),
            kind,
            chip: None,
            pin: None,
            data: None,
            src: None,
            rot: 0,
            label: None,
            x: 0.0,
            y: 0.0,
        }
    }

    /// A chip part of component `chip`.
    pub fn chip(id: &str, chip: &str) -> Part {
        Part {
            chip: Some(chip.into()),
            ..Part::new(id, PartKind::Chip)
        }
    }

    /// An input or output pin numbered `pin`, optionally named.
    pub fn pin(id: &str, kind: PartKind, pin: usize, label: Option<&str>) -> Part {
        Part {
            pin: Some(pin),
            label: label.map(String::from),
            ..Part::new(id, kind)
        }
    }
}

impl Circuit {
    /// Wire pin `a.1` of part `a.0` to pin `b.1` of part `b.0`.
    pub fn connect(&mut self, a: (&str, usize), b: (&str, usize)) {
        self.wires.push(Wire {
            from: End {
                part: a.0.into(),
                pin: a.1,
            },
            to: End {
                part: b.0.into(),
                pin: b.1,
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_saved_and_does_not_change_the_wiring() {
        let mut p = Part::chip("x", "nand");
        p.rot = 3;
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"rot\":3"));
        let back: Part = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rot, 3);
        assert!(
            !serde_json::to_string(&Part::chip("y", "nand"))
                .unwrap()
                .contains("rot"),
            "rot 0 is omitted"
        );
        let (mut a, mut b) = (Circuit::default(), Circuit::default());
        a.parts.push(Part::chip("x", "nand"));
        b.parts.push(back);
        assert_eq!(a.topology_key(), b.topology_key());
    }
}
