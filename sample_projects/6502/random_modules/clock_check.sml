{
  "connections": [
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 0,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 1,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 5,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 0,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 6,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 5,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 6,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 2,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 0,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 6,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 4,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 6,
        "port_id": null
      }
    }
  ],
  "next_id": 7,
  "parts": {
    "0": {
      "color": [
        44,
        230,
        230,
        255
      ],
      "id": 0,
      "label": "XOR",
      "part_data": {
        "Gate": {
          "gate_type": "Xor",
          "important": false
        }
      },
      "pos": {
        "x": 2160.0,
        "y": 160.0
      },
      "simulation_index": null
    },
    "1": {
      "color": [
        226,
        219,
        19,
        255
      ],
      "id": 1,
      "label": "start clock",
      "part_data": {
        "IO": {
          "input": true
        }
      },
      "pos": {
        "x": 2040.0,
        "y": 160.0
      },
      "simulation_index": null
    },
    "2": {
      "color": [
        226,
        219,
        19,
        255
      ],
      "id": 2,
      "label": "finish clock",
      "part_data": {
        "IO": {
          "input": true
        }
      },
      "pos": {
        "x": 2040.0,
        "y": 240.0
      },
      "simulation_index": null
    },
    "4": {
      "color": [
        226,
        219,
        19,
        255
      ],
      "id": 4,
      "label": "finish clock",
      "part_data": {
        "IO": {
          "input": false
        }
      },
      "pos": {
        "x": 2280.0,
        "y": 240.0
      },
      "simulation_index": null
    },
    "5": {
      "color": [
        117,
        20,
        237,
        255
      ],
      "id": 5,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "important": false
        }
      },
      "pos": {
        "x": 2280.0,
        "y": 160.0
      },
      "simulation_index": null
    },
    "6": {
      "color": [
        25,
        231,
        83,
        255
      ],
      "id": 6,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "important": false
        }
      },
      "pos": {
        "x": 2160.0,
        "y": 240.0
      },
      "simulation_index": null
    }
  }
}