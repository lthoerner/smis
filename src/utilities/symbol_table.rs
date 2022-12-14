// Represents the symbol table, which is a collection of all the Labels in the program
pub struct SymbolTable {
    labels: Vec<Label>
}

// Creates an empty SymbolTable
pub fn new() -> SymbolTable {
    SymbolTable { labels: Vec::new() }
}

impl SymbolTable {
    // Adds a label to the symbol table
    pub fn add_label(&mut self, name: &str, address: u16) {
        let name = name.to_string();

        let lbl = Label {
            name,
            address
        };

        self.labels.push(lbl);
    }
}

// Represents a label, which is a name given to an address in instruction memory
struct Label {
    name: String,
    address: u16
}