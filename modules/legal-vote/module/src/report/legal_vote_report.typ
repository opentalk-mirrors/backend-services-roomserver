// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

#set page(
  paper: "a4",
)
#set text(
  size: 10pt,
)

#let data = json("data.json")
#let parse_datetime(s) = toml(bytes("date = " + s)).date
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]:[second]"
#let vote_kind = (
  pseudonymous: "Pseudonymous vote",
  roll_call: "Roll call",
  live_roll_call: "Live roll call",
)
#let vote_option = (
  yes: "Yes",
  no: "No",
  abstain: "Abstain",
)

= OpenTalk Vote Report

#let metadata_table_content = (
  (
    [Title],
    data.summary.title,
  ),
)

#if "subtitle" in data.summary {
  metadata_table_content.push((
    [Subtitle],
    data.summary.subtitle,
  ))
}

#if "topic" in data.summary {
  metadata_table_content.push((
    [Topic],
    data.summary.topic,
  ))
}

#metadata_table_content.push((
  [Pseudonymous],
  [#if data.summary.pseudonymous { "Yes" } else { "No" }],
))

#metadata_table_content.push((
  [Referendum leader],
  data.summary.creator,
))

#metadata_table_content.push((
  [Vote id],
  data.summary.id,
))

#metadata_table_content.push((
  [Start],
  [ #parse_datetime(data.summary.start_time).display(datetime_format) ],
))

#if "end_time" in data.summary {
  metadata_table_content.push((
    [End],
    [ #parse_datetime(data.summary.end_time).display(datetime_format) ],
  ))
}

#metadata_table_content.push((
  [Report timezone],
  data.summary.report_timezone,
))

#metadata_table_content.push((
  [Participant count],
  data.summary.participant_count,
))

#metadata_table_content.push((
  [Scheduled duration],
  if "duration" in data.summary {
    [#data.summary.duration s]
  } else {
    [Unlimited]
  },
))

#metadata_table_content.push((
  [Abstention],
  if data.summary.enable_abstain {
    [Allowed]
  } else {
    [Disallowed]
  },
))

#metadata_table_content.push((
  [Automatic close],
  if data.summary.auto_close {
    [Enabled]
  } else {
    [Disabled]
  },
))

#metadata_table_content.push((
  [Vote ended due to],
  if data.summary.stop_reason.kind == "by_user" [
    User *#data.summary.stop_reason.user* ended the vote
  ] else if data.summary.stop_reason.kind == "auto" [
    All users voted
  ] else if data.summary.stop_reason.kind == "expired" [
    Expired
  ] else if data.summary.stop_reason.kind == "canceled" {
    if data.summary.stop_reason.reason == "room_destroyed" [
      Aborted by room being closed
    ] else if data.summary.stop_reason.reason == "initiator_left" [
      Aborted by vote initiator leaving
    ] else if data.summary.stop_reason.reason == "custom" [
      Aborted for custom reason: #data.summary.stop_reason.custom
    ] else [
      Aborted for unknown reason
    ]
  } else [
    Unknown reason
  ],
))

#metadata_table_content.push((
  [Number of votes],
  data.summary.vote_count,
))

#table(
  stroke: none,
  columns: 2,
  ..for (name, content) in metadata_table_content {
    ([*#name*:], [#content])
  }
)

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#if "final_results" in data.summary and data.summary.final_results.results == "valid" [

  == Results

  #let results_table_content = (
    (
      [Yes],
      data.summary.final_results.yes,
    ),
    (
      [No],
      data.summary.final_results.no,
    ),
  )

  #if "abstain" in data.summary.final_results {
    results_table_content.push((
      [Abstain],
      data.summary.final_results.abstain,
    ))
  }

  #set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
  #table(
    stroke: none,
    columns: (auto, 1fr),
    table.header([*Vote*], [*Count*]),
    table.hline(y: 0),
    table.hline(y: 1),
    ..for (vote, count) in results_table_content {
      ([#vote], [#count])
    },
  )

]

== Recorded votes

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
#table(
  stroke: none,
  columns: (auto, auto, auto, 1fr),
  table.header([*Name*], [*Token*], [*Vote*], [*Timestamp*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for vote in data.votes {
    (
      if "name" in vote [
        #vote.name
      ] else [
        Hidden
      ],
      [#vote.token],
      [#vote_option.at(vote.option)],
      if "time" in vote [
        #parse_datetime(vote.time).display(datetime_format)
      ] else [
        —
      ],
    )
  },
)

== Event log

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
#table(
  stroke: none,
  columns: (auto, auto, 1fr),
  table.header([*Name*], [*Timestamp*], [*Event*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for event in data.events {
    (
      if "name" in event.event_details [
        #event.event_details.name
      ] else [
        Anonymous
      ],
      if "time" in event [
        #parse_datetime(event.time).display(datetime_format)
      ] else [
        —
      ],
      {
        if event.kind == "issue" {
          let issue = if "kind" not in event.event_details [
            Reports a problem
          ] else if event.event_details.kind == "audio" [
            Reports an audio issue
          ] else if event.event_details.kind == "video" [
            Reports a video issue
          ] else if event.event_details.kind == "screenshare" [
            Reports a screenshare issue
          ]

          if "description" in event.event_details [
            #issue: #event.event_details.description
          ] else [
            #issue
          ]
        } else if event.kind == "user_joined" [
          User joined
        ] else if event.kind == "user_left" [
          User left
        ]
      },
    )
  },
)
