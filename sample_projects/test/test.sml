{
  "connections": [
    {
      "end": {
        "input": true,
        "part": 5,
        "port_id": null
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 1,
        "port_id": 7
      }
    },
    {
      "end": {
        "input": true,
        "part": 1,
        "port_id": 3
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 2,
        "port_id": null
      }
    },
    {
      "end": {
        "input": true,
        "part": 1,
        "port_id": 4
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
        "part": 1,
        "port_id": 5
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
        "part": 1,
        "port_id": 6
      },
      "powered": false,
      "start": {
        "input": false,
        "part": 5,
        "port_id": null
      }
    }
  ],
  "next_id": 7,
  "parts": {
    "1": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 1,
      "label": "module2",
      "part_data": {
        "Module": {
          "canvas_snapshot": {
            "connections": [
              {
                "end": {
                  "input": true,
                  "part": 2,
                  "port_id": null
                },
                "powered": false,
                "start": {
                  "input": false,
                  "part": 0,
                  "port_id": null
                }
              },
              {
                "end": {
                  "input": true,
                  "part": 2,
                  "port_id": null
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
                  "part": 1,
                  "port_id": null
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
                  "port_id": null
                },
                "powered": false,
                "start": {
                  "input": false,
                  "part": 5,
                  "port_id": null
                }
              },
              {
                "end": {
                  "input": true,
                  "part": 1,
                  "port_id": null
                },
                "powered": false,
                "start": {
                  "input": false,
                  "part": 6,
                  "port_id": null
                }
              },
              {
                "end": {
                  "input": true,
                  "part": 7,
                  "port_id": null
                },
                "powered": false,
                "start": {
                  "input": false,
                  "part": 2,
                  "port_id": null
                }
              }
            ],
            "next_id": 8,
            "parts": {
              "0": {
                "color": [
                  223,
                  127,
                  1,
                  255
                ],
                "id": 0,
                "label": "AND",
                "part_data": {
                  "Gate": {
                    "gate_type": "And",
                    "powered": false
                  }
                },
                "pos": {
                  "x": 420.0,
                  "y": 320.0
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
                  "x": 420.0,
                  "y": 420.0
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
                "label": "OR",
                "part_data": {
                  "Gate": {
                    "gate_type": "Or",
                    "powered": false
                  }
                },
                "pos": {
                  "x": 600.0,
                  "y": 380.0
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
                "label": "Input",
                "part_data": {
                  "IO": {
                    "input": true
                  }
                },
                "pos": {
                  "x": 200.0,
                  "y": 280.0
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
                "label": "Input",
                "part_data": {
                  "IO": {
                    "input": true
                  }
                },
                "pos": {
                  "x": 200.0,
                  "y": 340.0
                }
              },
              "5": {
                "color": [
                  223,
                  127,
                  1,
                  255
                ],
                "id": 5,
                "label": "Input",
                "part_data": {
                  "IO": {
                    "input": true
                  }
                },
                "pos": {
                  "x": 200.0,
                  "y": 400.0
                }
              },
              "6": {
                "color": [
                  223,
                  127,
                  1,
                  255
                ],
                "id": 6,
                "label": "Input",
                "part_data": {
                  "IO": {
                    "input": true
                  }
                },
                "pos": {
                  "x": 200.0,
                  "y": 460.0
                }
              },
              "7": {
                "color": [
                  223,
                  127,
                  1,
                  255
                ],
                "id": 7,
                "label": "Output",
                "part_data": {
                  "IO": {
                    "input": false
                  }
                },
                "pos": {
                  "x": 800.0,
                  "y": 380.0
                }
              }
            }
          },
          "inputs": {
            "3": "Input",
            "4": "Input",
            "5": "Input",
            "6": "Input"
          },
          "min_size": {
            "x": 80.0,
            "y": 120.0
          },
          "outputs": {
            "7": "Output"
          },
          "path": "module2.sml",
          "size": {
            "x": 120.0,
            "y": 120.0
          }
        }
      },
      "pos": {
        "x": 360.0,
        "y": 400.0
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
        "x": 200.0,
        "y": 360.0
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
        "x": 200.0,
        "y": 420.0
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
        "x": 200.0,
        "y": 480.0
      }
    },
    "5": {
      "color": [
        223,
        127,
        1,
        255
      ],
      "id": 5,
      "label": "AND",
      "part_data": {
        "Gate": {
          "gate_type": "And",
          "powered": false
        }
      },
      "pos": {
        "x": 520.0,
        "y": 540.0
      }
    }
  }
}