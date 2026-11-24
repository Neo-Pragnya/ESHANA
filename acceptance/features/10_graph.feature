@cap_graph
Feature: Symbolic graph

  Scenario: JSON graph has typed nodes and relation classes and no causal edges
    When I run "eshana graph vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 9"
    Then it succeeds
    And the JSON output at "kind" equals "symbolic_graph"
    And the output contains "traditional_correspondence"
    And the output contains ""relation_class": "computational""
    And the output does not contain ""relation_class": "causal""
    And the output contains "No edge asserts a causal link"

  Scenario: Mermaid output pipes straight into a renderer
    When I run "eshana graph vedic --utc 2000-01-01T12:00:00Z --format mermaid"
    Then it succeeds
    And the output contains "flowchart LR"
    And the output contains "classDef calculated"

  Scenario: GraphML is well-formed XML
    Given the file "wellformed.py" with content:
      """
      import sys
      import xml.dom.minidom as m
      doc = m.parse(sys.argv[1])
      print("well-formed", len(doc.getElementsByTagName("node")), "nodes", len(doc.getElementsByTagName("edge")), "edges")
      """
    When I run "eshana graph western --utc 2000-01-01T12:00:00Z --zodiac tropical --format graphml > g.graphml && python wellformed.py g.graphml"
    Then it succeeds
    And the output contains "well-formed"

  Scenario: SVG and draw.io exports are well-formed drawings
    Given the file "wellformed.py" with content:
      """
      import sys
      import xml.dom.minidom as m
      doc = m.parse(sys.argv[1])
      print("root", doc.documentElement.tagName, "boxes", len(doc.getElementsByTagName("rect")) + len(doc.getElementsByTagName("mxCell")))
      """
    When I run "eshana graph vedic --utc 2000-01-01T12:00:00Z --format svg > g.svg && python wellformed.py g.svg && eshana graph vedic --utc 2000-01-01T12:00:00Z --format drawio > g.drawio && python wellformed.py g.drawio"
    Then it succeeds
    And the output contains "root svg"
    And the output contains "root mxfile"

  Scenario: The Chitkala export carries the nodes, edges and sources
    When I run "eshana graph vedic --utc 2000-01-01T12:00:00Z --format chitkala"
    Then it succeeds
    And the JSON output at "nodes" has at least 10 items
    And the JSON output at "edges" has at least 10 items
    And the output contains "universes"

  Scenario: An unknown format is refused
    When I run "eshana graph vedic --utc 2000-01-01T12:00:00Z --format visio"
    Then the exit code is 2
    And the output contains "ESH-INPUT"
