// Represents the symbol table, which is a collection of all the Labels in the program
pub struct SymbolTable {
    labels: Vec<Label>
}

// Represents a label, which is a name given to an address in instruction memory
struct Label {
    name: &str,
    address: u16
}