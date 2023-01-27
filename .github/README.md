---
feature: phil-wadler
authors:
  - "ice1000"
start_date: "2022/12/01"
---

# Wadler-style algebraic pretty printing API for SQL

This RFC proposes a new API for pretty-printing pseudo "structures".
The purpose of the API is to supersede the current implementation of SQL explain,
and maybe more.

## Goals

+ Make the output of SQL explain "cooler" in the sense that ASCII (or Unicode) art
  are used to help with the readability of the output.
+ Implementation-wise, the API should be extensible and not tightly coupled with
  the actual SQL syntax so that it can be used for other purposes.
+ The current implementation is going to be replaced by the new API, if things went well.

## Non-goals

+ The new API is not designed for performance-critical applications.
  SQL explain is not considered to be such an application.
+ The new API does not aim to be super flexible like the `pretty` crate.
  This gives us spaces to simplify the design and implementation.

## Motivation

I tried to use the `pretty` crate to implement SQL explain, but it turned out to be limited in many ways:

+ The standard Wadler-style pretty printing API only controls lines, indentation, text wrapping, etc.
  which is not suitable for sophisticated insertion of the box-making or table-making characters.
+ It does not support "wrapping" the output in any ways. We want to make a big "box" around the output.
+ It does not support the tree-making characters, which are essential but requires a stack in the doc-to-string algorithm.
  The standard implementation of `pretty` is a pure `(Config, Doc) -> String` algorithm with potential configurations.
  I believe that we essentially need to upgrade this from a reader monad to a state monad.
+ It supports horizontal and vertical "squeezing" of the output (say, limit the max column/line numbers,
  and try to fit in by inserting/removing new lines), but we only need horizontal squeezing.

However, the standard Wadler-style "algebraic" pretty printing API is well-designed and can be extended to support the features we desire.
I saw a screenshot by `@xxchan` on a private Slack channel that shows the SQL explain output of databend's system,
which inspired me to write this RFC.

## Intended behavior

+ Users specify a preferred width, usually the width of the terminal,
  or 80 or 120, etc.
+ The API automatically calculates the actual width of the output,
  based on the preferred width.
  + If everything can be done in one line, the actual width is the line's width, and the output will be one-linear.
  + If the output cannot be done in one line, the output will try to break down the output into multiple lines, and retry to fit the output into the preferred width for every line.
+ The API supports wrapping the output with beautiful ASCII/Unicode art.

## Implementation

These contents are subject to future changes.

### Types

#### Types `XmlNode` and `Pretty` for pretty printing data

+ These enums are inductive-inductively defined which represents an object that can be displayed as a string.
+ The width and height of the pretty-printed string can be calculated in advance.
+ Instances of the enum `Pretty` are hereafter called "pretty" or "pretties".
+ Instances of struct `XmlNode` represent XML-like data that has a name, a list of attributes, and a list of children nodes.

Variants of `Pretty`:

+ Variant `Record` that brutally pretty-prints an XML-like data.
  + It contains an XML node.
+ Variant `Array` that brutally pretty-prints an array-like data.
  + It contains a list of pretties.
+ Variant `Text` that pretty-prints a string.
  + It contains a copy-on-write string.

#### Record `PrettyConfig` for pretty printing configuration

It contains indentation, preferred width, etc.

#### Record `LinedBuffer` for actually writing the string

It contains a mutable reference to a `String`, and a `PrettyConfig`.
It understands the intended width (precomputed by `PrettyConfig::interesting_*`),
and will try to fill an incomplete line with spaces when asked so.

### Important methods

+ `Pretty::ol_len_*(&self) -> usize`
  + Returns the length of the pretty-printed string, under a one-linear setting.
+ `Pretty::ol_build_string_*(&self, build: &mut String)`
  + Builds the pretty-printed string, under a one-linear setting.
+ `PrettyConfig::interesting_*`
  + Predicts the width and the total length of the pretty-printed string.
+ `LinedBuffer::line_*` (private)
  + Generate a line, **without** the starting `|` and the ending `|` and the indentations.
    It will try to fill the intermediate spaces and lines, but not the surrounding.
+ `PrettyConfig::horizon`
  + Generates a line of a given length with `+` at the ends and `-` in the middle.
+ `PrettyConfig::ascii`
  + Calls `interesting` to predict the output width, and then generate the beautiful output, using pure ASCII style.
+ `PrettyConfig::unicode`
  + Calls `interesting` to predict the output width, and then generate the beautiful output, using Unicode table-making characters.

### Edge cases

+ All methods handle empty lists and no-children records. No-field records are not tested yet.
