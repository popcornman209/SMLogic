{
  "connections": [
    {
      "end": {
        "input": true,
        "part": 3,
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
        "part": 3,
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
        "part": 4,
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
        "part": 4,
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
        "part": 7,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 4,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 7,
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
        "part": 8,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 4,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 8,
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
        "part": 11,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 8,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 11,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 3,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 13,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 11,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 12,
        "port_id": null
      },
      "start": {
        "input": false,
        "part": 7,
        "port_id": null
      }
    }
  ],
  "next_id": 14,
  "parts": {
    "0": {
      "color": [
        10,
        62,
        226,
        255
      ],
      "id": 0,
      "label": "Input 2",
      "part_data": {
        "IO": {
          "input": true
        }
      },
      "pos": {
        "x": 580.0,
        "y": 460.0
      },
      "simulation_index": null
    },
    "1": {
      "color": [
        117,
        20,
        237,
        255
      ],
      "id": 1,
      "label": "carry",
      "part_data": {
        "IO": {
          "input": true
        }
      },
      "pos": {
        "x": 580.0,
        "y": 520.0
      },
      "simulation_index": null
    },
    "11": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 11,
      "label": "OR",
      "part_data": {
        "Gate": {
          "gate_type": "Or",
          "important": false
        }
      },
      "pos": {
        "x": 840.0,
        "y": 400.0
      },
      "simulation_index": null
    },
    "12": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 12,
      "label": "Output",
      "part_data": {
        "IO": {
          "input": false
        }
      },
      "pos": {
        "x": 960.0,
        "y": 460.0
      },
      "simulation_index": null
    },
    "13": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 13,
      "label": "carry",
      "part_data": {
        "IO": {
          "input": false
        }
      },
      "pos": {
        "x": 960.0,
        "y": 520.0
      },
      "simulation_index": null
    },
    "2": {
      "color": [
        44,
        230,
        230,
        255
      ],
      "id": 2,
      "label": "input 1",
      "part_data": {
        "IO": {
          "input": true
        }
      },
      "pos": {
        "x": 580.0,
        "y": 400.0
      },
      "simulation_index": null
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
          "important": false
        }
      },
      "pos": {
        "x": 700.0,
        "y": 400.0
      },
      "simulation_index": null
    },
    "4": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 4,
      "label": "XOR",
      "part_data": {
        "Gate": {
          "gate_type": "Xor",
          "important": false
        }
      },
      "pos": {
        "x": 700.0,
        "y": 460.0
      },
      "simulation_index": null
    },
    "7": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 7,
      "label": "XOR",
      "part_data": {
        "Gate": {
          "gate_type": "Xor",
          "important": false
        }
      },
      "pos": {
        "x": 820.0,
        "y": 560.0
      },
      "simulation_index": null
    },
    "8": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 8,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "important": false
        }
      },
      "pos": {
        "x": 820.0,
        "y": 500.0
      },
      "simulation_index": null
    }
  }
}