//! # Paragraph
//! 
//! The `paragraph` module defines the `Paragraph` struct, which represents a single paragraph in a PBV2 document. It contains the text of the paragraph and any associated formatting information.
//! 
//! The `Paragraph` contains a vector of `ParagraphElement`s, which can be either plain text or a reference to another element that is inline with the text. This allows for rich formatting and the inclusion of various types of content within a paragraph.