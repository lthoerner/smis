// Represents the symbol table, which is a collection of all the Labels in the program
#[derive(Debug, Default)]
pub struct SymbolTable {
    labels: Vec<Label>,
}

// Represents a label, which is a name given to an address in instruction memory
#[derive(Debug)]
pub struct Label {
    name: String,
    address: u16,
}

impl SymbolTable {
    // Adds a label to the symbol table
    pub fn add_label(&mut self, name: &str, address: u16) {
        self.labels.push(Label {
            name: name.to_owned(),
            address,
        });
    }

    // Gets the corresponding label address in the symbol table for a given label name
    pub fn find_address(&self, requested_name: &str) -> Option<u16> {
        for label in &self.labels {
            if label.name == requested_name {
                return Some(label.address);
            }
        }

        None
    }

    // Gets the corresponding label name in the symbol table for a given label address
    pub fn find_name(&self, requested_address: u16) -> Option<String> {
        for label in &self.labels {
            if label.address == requested_address {
                return Some(label.name.clone());
            }
        }

        None
    }

    // Checks if the symbol table already contains a label with the given address
    pub fn contains(&self, address: u16) -> bool {
        self.find_name(address).is_some()
    }
}
