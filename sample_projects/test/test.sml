{
  "connections": [
    {
      "end": {
        "input": true,
        "part": 3,
        "port_id": null
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 0,
        "port_id": 7
      }
    },
    {
      "end": {
        "input": true,
        "part": 4,
        "port_id": null
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 3,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": 3
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 4,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": 4
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 1,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": 6
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 3,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": 5
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 2,
        "port_id": null
      }
    }
  ],
  "next_id": 5,
  "parts": {
    "0": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 0,
      "label": "module2",
      "part_data": {
        "Module": {
          "canvas_snapshot": {
            "connections": [],
            "next_id": 0,
            "parts": {}
          },
          "inputs": [
            [
              4,
              "Input"
            ],
            [
              5,
              "Input"
            ],
            [
              6,
              "Input"
            ],
            [
              3,
              "Input"
            ]
          ],
          "min_size": {
            "x": 80.0,
            "y": 120.0
          },
          "outputs": [
            [
              7,
              "Output"
            ]
          ],
          "path": "module2.sml",
          "size": {
            "x": 120.0,
            "y": 120.0
          }
        }
      },
      "pos": {
        "x": 440.0,
        "y": 180.0
      }
    },
    "1": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 1,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "powered": false
        }
      },
      "pos": {
        "x": 240.0,
        "y": 180.0
      }
    },
    "2": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 2,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "powered": false
        }
      },
      "pos": {
        "x": 240.0,
        "y": 240.0
      }
    },
    "3": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 3,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "powered": false
        }
      },
      "pos": {
        "x": 640.0,
        "y": 180.0
      }
    },
    "4": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 4,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "powered": false
        }
      },
      "pos": {
        "x": 640.0,
        "y": 260.0
      }
    }
  }
}