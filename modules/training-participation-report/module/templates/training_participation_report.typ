// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

#import "@preview/linguify:0.4.2": *

#set page(
  paper: "a4",
)
#set text(
  size: 10pt,
)

#let data = json("data.json")

#set-database(eval(load_ftl_data("./l10n", data.available_languages)))
#set text(lang: data.report_language)

#let parse_datetime(s) = toml(bytes("date = " + s)).date
#let time_format = "[hour]:[minute]"
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]"

= #linguify("training_participation_report")

#table(
  stroke: none,
  columns: 2,
  [*#linguify("header_meeting")*], [#data.title],
  [*#linguify("header_description")*], if data.description.len() > 0 [#data.description] else [—],
  [*#linguify("header_report_timezone")*], [#data.report_timezone],
  [*#linguify("header_training_start")*], [#parse_datetime(data.start).display(datetime_format)],
  [*#linguify("header_training_end")*], [#parse_datetime(data.end).display(datetime_format)],
)

#let checkpoints_per_table = 8

#let data_tables = ()

#let chunks = int(data.checkpoints.len() / checkpoints_per_table)
#if calc.rem(data.checkpoints.len(), checkpoints_per_table) != 0 {
  chunks += 1
}

/// Insert a zero-width space into a word after a certain length.
/// This avoids breaking words shorter than the `after` length.
#let insert_zero_width_space_after(s, after: int) = {
  let codepoints = s.codepoints()
  let result = ""
  for i in range(0, codepoints.len()) {
    if i < after {
      result += codepoints.at(i)
    } else {
      result += sym.zws + codepoints.at(i)
    }
  }
  result
}

/// Insert zero-width spaces into each word of a text after a certain
/// word length. Allows breaking long words instead of breaking inside
/// short words within a text that exceeds a certain length.
#let make_long_words_breakable(s, after: int) = {
  s.split(" ").map(word => insert_zero_width_space_after(word, after: after)).join(" ")
}

== #linguify("participation_checkpoints")

#for i in range(0, chunks) {
  let offset = i * checkpoints_per_table
  let chunk_size = if (offset + checkpoints_per_table) > data.checkpoints.len() {
    data.checkpoints.len() - offset
  } else {
    checkpoints_per_table
  }
  let checkpoints = data.checkpoints.slice(offset, count: chunk_size)
  let column_count = checkpoints.len()
  let header = (
    align(end)[*#linguify("nr")*],
    [*#linguify("person")*],
    ..checkpoints.map(checkpoint => [
      #align(center)[*#parse_datetime(checkpoint.timestamp).display(time_format)*]
    ]),
  )
  let columns = (2em, 15em)
  for i in range(0, checkpoints_per_table) {
    columns.push(1fr)
  }

  let rows = ()
  for (number, (id, name)) in data.participants.pairs().sorted(key: k => k.at(1)).enumerate(start: 1) {
    let row = (
      align(end)[#number],
      if name == none [
        _#linguify("unknown")_
      ] else [
        *#make_long_words_breakable(name, after: 15)*
      ],
      checkpoints.map(checkpoint => if id in checkpoint.presence [
        #align(center)[#parse_datetime(checkpoint.presence.at(id)).display(time_format)]
      ] else [
        #align(center)[—]
      ]),
      range(0, checkpoints_per_table - checkpoints.len()).map(i => []),
    )
    rows.push(row)
  }
  data_tables.push((header: header, rows: rows, columns: columns))
}

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#for data_table in data_tables {
  table(
    stroke: none,
    columns: data_table.columns,
    table.hline(y: 0),
    table.hline(y: 1),
    table.header(..data_table.header),
    ..data_table.rows.flatten()
  )
}
