// got these from 'overboard'
// https://twitter.com/jcreed/status/851457146852184065
// beepbox export to json then
// jq -c '[.channels[0,1,2].patterns[1].notes[]|{pitch:.pitches[0], time: [.points[0].tick, .points[1].tick]}]'  /tmp/BeepBox-Song.json

import { Score } from './types';
export const score: Score = {
  "loop_start": 0,
  "loop_end": 32,
  "next_id": 271,
  "duration": 65,
  "seconds_per_tick": 0.09,
  "song": [
    {
      "lane": 1,
      "patName": "default",
      "start": 0,
      "duration": 64
    },
    {
      "lane": 0,
      "patName": "drums",
      "start": 0,
      "duration": 64
    }
  ],
  "patterns": {
    "drums": {
      "length": 32,
      "notes": [
        {
          "pitch": 57,
          "time": [
            4,
            6
          ],
          "id": "n188"
        },
        {
          "pitch": 57,
          "time": [
            12,
            14
          ],
          "id": "n190"
        },
        {
          "pitch": 57,
          "time": [
            20,
            22
          ],
          "id": "n192"
        },
        {
          "pitch": 57,
          "time": [
            28,
            30
          ],
          "id": "n193"
        },
        {
          "pitch": 52,
          "time": [
            0,
            2
          ],
          "id": "n195"
        },
        {
          "pitch": 52,
          "time": [
            10,
            12
          ],
          "id": "n197"
        },
        {
          "pitch": 52,
          "time": [
            16,
            18
          ],
          "id": "n198"
        },
        {
          "pitch": 52,
          "time": [
            26,
            28
          ],
          "id": "n199"
        },
        {
          "pitch": 61,
          "time": [
            0,
            2
          ],
          "id": "n200"
        },
        {
          "pitch": 61,
          "time": [
            4,
            5
          ],
          "id": "n201"
        },
        {
          "pitch": 61,
          "time": [
            8,
            10
          ],
          "id": "n202"
        },
        {
          "pitch": 61,
          "time": [
            12,
            14
          ],
          "id": "n203"
        },
        {
          "pitch": 61,
          "time": [
            16,
            18
          ],
          "id": "n204"
        },
        {
          "pitch": 61,
          "time": [
            20,
            21
          ],
          "id": "n205"
        },
        {
          "pitch": 61,
          "time": [
            24,
            25
          ],
          "id": "n206"
        },
        {
          "pitch": 61,
          "time": [
            28,
            29
          ],
          "id": "n207"
        },
        {
          "pitch": 52,
          "time": [
            14,
            16
          ],
          "id": "n211"
        },
        {
          "pitch": 61,
          "time": [
            10,
            11
          ],
          "id": "n226"
        },
        {
          "pitch": 61,
          "time": [
            11,
            12
          ],
          "id": "n227"
        },
        {
          "pitch": 61,
          "time": [
            14,
            15
          ],
          "id": "n229"
        },
        {
          "pitch": 61,
          "time": [
            15,
            16
          ],
          "id": "n230"
        },
        {
          "pitch": 61,
          "time": [
            18,
            20
          ],
          "id": "n231"
        },
        {
          "pitch": 61,
          "time": [
            22,
            24
          ],
          "id": "n232"
        },
        {
          "pitch": 61,
          "time": [
            26,
            28
          ],
          "id": "n233"
        },
        {
          "pitch": 61,
          "time": [
            29,
            31
          ],
          "id": "n234"
        },
        {
          "pitch": 61,
          "time": [
            31,
            32
          ],
          "id": "n236"
        },
        {
          "pitch": 61,
          "time": [
            2,
            4
          ],
          "id": "n237"
        },
        {
          "pitch": 61,
          "time": [
            6,
            8
          ],
          "id": "n239"
        },
        {
          "pitch": 61,
          "time": [
            5,
            6
          ],
          "id": "n240"
        }
      ]
    },
    "default": {
      "length": 32,
      "notes": [
        {
          "pitch": 56,
          "time": [
            0,
            2
          ],
          "id": "n241"
        },
        {
          "pitch": 56,
          "time": [
            3,
            5
          ],
          "id": "n242"
        },
        {
          "pitch": 56,
          "time": [
            6,
            8
          ],
          "id": "n243"
        },
        {
          "pitch": 59,
          "time": [
            8,
            10
          ],
          "id": "n244"
        },
        {
          "pitch": 59,
          "time": [
            11,
            13
          ],
          "id": "n245"
        },
        {
          "pitch": 59,
          "time": [
            14,
            16
          ],
          "id": "n246"
        },
        {
          "pitch": 61,
          "time": [
            18,
            20
          ],
          "id": "n247"
        },
        {
          "pitch": 61,
          "time": [
            21,
            23
          ],
          "id": "n248"
        },
        {
          "pitch": 59,
          "time": [
            24,
            26
          ],
          "id": "n253"
        },
        {
          "pitch": 59,
          "time": [
            27,
            29
          ],
          "id": "n254"
        },
        {
          "pitch": 59,
          "time": [
            30,
            32
          ],
          "id": "n255"
        },
        {
          "pitch": 51,
          "time": [
            16,
            18
          ],
          "id": "n257"
        },
        {
          "pitch": 51,
          "time": [
            20,
            21
          ],
          "id": "n258"
        },
        {
          "pitch": 51,
          "time": [
            23,
            24
          ],
          "id": "n259"
        },
        {
          "pitch": 68,
          "time": [
            2,
            3
          ],
          "id": "n260"
        },
        {
          "pitch": 68,
          "time": [
            5,
            6
          ],
          "id": "n261"
        },
        {
          "pitch": 71,
          "time": [
            13,
            14
          ],
          "id": "n263"
        },
        {
          "pitch": 71,
          "time": [
            10,
            11
          ],
          "id": "n264"
        },
        {
          "pitch": 63,
          "time": [
            20,
            21
          ],
          "id": "n266"
        },
        {
          "pitch": 68,
          "time": [
            20,
            30
          ],
          "id": "n269"
        },
        {
          "pitch": 67,
          "time": [
            30,
            32
          ],
          "id": "n270"
        }
      ]
    }
  }
}
