/// Flags used to open a file. May be incomplete
pub enum OpenFlags {
    ORDO,   // read-only
    OWRO,   // write-only
    ORDWR,  // read and write
    OCREAT, // Create the file if doesn't exist
}
