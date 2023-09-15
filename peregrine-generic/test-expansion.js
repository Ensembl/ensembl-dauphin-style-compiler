/* 
This script contains some test code for expansion tracks, 
taken from index.html to reduce clutter. May be useful in future development.
*/
// Expansion track test code
genome_browser.add_jsapi_channel("test", {
  callbacks: {
    jump: function (location) {
      return delay(100).then(() => {
        let parts = location.split(":");
        let stick = parts.slice(1, parts.length - 1).join(":");
        if (parts.length < 3 || parts[0] != "region") { return null; }
        let start_end = parts[parts.length - 1].split("-");
        if (start_end.length < 2) { return null; }
        return {
          "stick": stick,
          "left": parseInt(start_end[0]),
          "right": parseInt(start_end[1])
        };
      })
    },
    data: function (source, stick, scale, index, scope) {
      console.log("data " + source + " " + stick + " " + scale + " " + index + " " + scope);
      return {
        "data": {
          "start": [86200000, 86400000],
          "end": [86300000, 86500000]
        }
      };
    },
    small_values: function (namespace, column) {
      return {};
    },
    stickinfo: function (stick) {
      return delay(100).then(() => {
        console.log("stick " + stick);
        return {};
      })
    },
    boot: function () {
      return delay(100).then(() => {
        console.log("booting inside", this);
        let expansions = [
          {
            "name": "js-expansion",
            "backend_namespace": this.backend_namespace,
            "triggers": [
              ["track", "user"]
            ]
          }
        ];
        let tracks = [
          {
            "program": ["special", "test", 1],
            "tags": "",
            "triggers": [
              ["track", "user2"]
            ],
            "settings": {
              "green": ["setting", "green"]
            },
            "values": {
              "red": 100.0,
              "blue": 0.0
            },
            "scale_start": 0,
            "scale_end": 60,
            "scale_step": 1
          }
        ];
        return { expansions, tracks };
      });
    },
    expand: function (name, step) {
      return delay(100).then(() => {
        if (name == "js-expansion") {
          let tracks = [
            {
              "program": ["ensembl-webteam/core", "test-with-data", 1],
              "tags": "",
              "triggers": [
                ["track", "user", step]
              ],
              "settings": {
                "green": ["settings", "green"]
              },
              "values": {
                "red": parseInt(step),
                "blue": 100.0
              },
              "scale_start": 0,
              "scale_end": 60,
              "scale_step": 1
            }
          ];
          console.log("expand", name, step);
          return { tracks };
        } else {
          return {};
        }
      });
    },
    program: function (group, name, version) {
      console.log("program", group, name, version);
      return {};
    }
  }
});

genome_browser.switch(["settings"],true);
genome_browser.switch(["settings","green"],50);
genome_browser.switch(["track","user"],true);
genome_browser.switch(["track", "user", "78"], true);

genome_browser.switch(["track","expand","ff0099"],true);

genome_browser.switch(["settings"],true);
genome_browser.switch(["settings","green"],50);
genome_browser.switch(["track","user"],true);
genome_browser.switch(["track","user","78"],true);
          
