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

#let parse_datetime(s) = toml.decode("date = " + s).date
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]"
#let role_label = (
  moderator: linguify("moderator"),
  user: linguify("user"),
  guest: linguify("guest"),
)
#let role_order = (
  moderator: 0,
  user: 1,
  guest: 2,
)

= #linguify("attendance_report")

#let metadata_table_content = (
  (
    linguify("meeting"),
    data.title,
  ),
)

#if data.description.len() > 0 {
  metadata_table_content.push((
    linguify("details"),
    data.description,
  ))
}

#if "starts_at" in data {
  metadata_table_content.push((
    linguify("planned_start"),
    parse_datetime(data.starts_at).display(datetime_format),
  ))
}

#if "ends_at" in data {
  metadata_table_content.push((
    linguify("planned_end"),
    parse_datetime(data.ends_at).display(datetime_format),
  ))
}

#metadata_table_content.push((
  linguify("report_created_at"),
  parse_datetime(data.report_created_at).display(datetime_format),
))

#metadata_table_content.push((
  linguify("report_timezone"),
  data.report_timezone,
))


#table(
  stroke: none,
  columns: 2,
  ..for (name, content) in metadata_table_content {
    ([*#name*:], [#content])
  }
)

== #linguify("participants")

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#table(
  stroke: none,
  columns: (auto, auto, 1fr),
  table.header([*#linguify("nr")*], [*#linguify("name")*], [*#linguify("role")*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for (i, participant) in data.participants.sorted(key: p => role_order.at(p.role)).enumerate(start: 1) {
    (
      [#i],
      [#participant.name],
      [#role_label.at(participant.role)],
    )
  },
)
