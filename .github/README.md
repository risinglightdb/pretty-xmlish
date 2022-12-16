---
feature: phil-wadler
authors:
  - "ice1000"
start_date: "2022/12/01"
---

# Wadler-style pretty printing API for SQL

This RFC proposes a new API for pretty-printing pseudo "structures".
The purpose of the API is to supersede the current implementation of SQL explain.

## Goals

+ Make the output of SQL explain "cooler" in the sense that ASCII (or Unicode) art
  are used to help with the readability of the output.
+ Implementation-wise, the API should be extensible and not tightly coupled with
  the actual SQL syntax so that it can be used for other purposes.
+ The current implementation is going to be replaced by the new API.

## Non-goals

+ The new API is not designed for performance-critical applications.
  SQL explain is not considered to be such an application.
+ The new API does not aim to be super flexible like the `pretty` crate.
  This gives us spaces to simplify the design and implementation.

## Types and traits

+ Enum `Pretty` for pretty printing.
  + It represents an object that can be displayed as a string.
  + The width and height of the pretty-printed string can be calculated in advance.
  + Objects that implement `Pretty` are hereafter called "pretty" or "pretties".
+ Struct `Record` that brutally pretty-prints a struct-like data.
  + It contains a list of name-pretty pairs.

