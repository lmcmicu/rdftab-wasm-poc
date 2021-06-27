import { initialise_thin, thin_to_thick, render_rows } from "rdftab-wasm-poc";

// Call the Rust WASM code to initialise an empty structure that will hold the thin rows:
let thin_rows = initialise_thin();
// Add the following rows:
thin_rows.by_stanza["ex:foo"] = [
  ["ex:foo", "ex:foo", "rdf:type", "owl:Class", "", "", ""],
  ["ex:foo", "_:riog00000002", "rdf:type", "rdf:List", "", "", ""],
  ["ex:foo", "_:riog00000002", "rdf:first", "", "A", "", ""],
  ["ex:foo", "_:riog00000003", "rdf:type", "rdf:List", "", "", ""],
  ["ex:foo", "_:riog00000003", "rdf:first", "", "B", "", ""],
  ["ex:foo", "_:riog00000003", "rdf:rest", "rdf:nil", "", "", ""],
  ["ex:foo", "_:riog00000002", "rdf:rest", "_:riog00000003", "", "", ""],
  ["ex:foo", "ex:foo", "ex:items", "_:riog00000002", "", "", ""],
  ["ex:foo", "ex:foo", "ex:link", "<https://example.com/FOO>", "", "", ""],
  ["ex:foo", "ex:foo", "ex:size", "", "123", "xsd:int", ""],
  ["ex:foo", "ex:foo", "rdfs:label", "", "Foo", "", ""],
  ["ex:foo", "ex:foo", "rdfs:label", "", "Fou", "", "fr"],
  ["ex:foo", "_:riog00000004", "rdf:type", "owl:Restriction", "", "", ""],
  ["ex:foo", "_:riog00000004", "owl:onProperty", "ex:part-of", "", "", ""],
  ["ex:foo", "_:riog00000004", "owl:someValuesFrom", "ex:bar", "", "", ""],
  ["ex:foo", "ex:foo", "rdfs:subClassOf", "_:riog00000004", "", "", ""],
  ["ex:foo", "_:riog00000005", "rdf:type", "owl:Axiom", "", "", ""],
  ["ex:foo", "_:riog00000005", "rdfs:comment", "", "OWL axiom annotation", "", "en"],
  ["ex:foo", "_:riog00000005", "owl:annotatedProperty", "ex:link", "", "", ""],
  ["ex:foo", "_:riog00000005", "owl:annotatedSource", "ex:foo", "", "", ""],
  ["ex:foo", "_:riog00000005", "owl:annotatedTarget", "<https://example.com/FOO>", "", "", ""],
  ["ex:foo", "_:riog00000006", "rdf:type", "rdf:Statement", "", "", ""],
  ["ex:foo", "_:riog00000006", "rdf:object", "<https://example.com/FOO>", "", "", ""],
  ["ex:foo", "_:riog00000006", "rdf:predicate", "ex:link", "", "", ""],
  ["ex:foo", "_:riog00000006", "rdf:subject", "ex:foo", "", "", ""],
  ["ex:foo", "_:riog00000006", "rdfs:comment", "", "RDF metadata", "", "en"]
];

// Call the Rust WASM code to convert the thin rows to thick rows:
let thick_rows = thin_to_thick(thin_rows);

// Note that the code to generate the thick triples table is loosely based on:
// https://www.encodedna.com/javascript/populate-json-data-to-html-table-using-javascript.htm
var col = ["subject", "predicate", "object", "value", "datatype", "language", "annotations",
           "metadata"];

var table = document.createElement("table");
// Add the header row:
var tr = table.insertRow(-1);
for (var i = 0; i < col.length; i++) {
  var th = document.createElement("th");
  th.innerHTML = col[i];
  tr.appendChild(th);
}

// Add the data rows:
for (var i = 0; i < thick_rows.length; i++) {
  tr = table.insertRow(-1);
  for (var j = 0; j < col.length; j++) {
    var cellData = thick_rows[i][col[j]];
    if (cellData && !(typeof(cellData) == "string")) {
      cellData = JSON.stringify(cellData, null, 2);
    } else if (cellData && cellData.startsWith("{")) {
      cellData = JSON.stringify(JSON.parse(cellData), null, 2);
    } else if (!cellData) {
      cellData = "";
    }
    var tabCell = tr.insertCell(-1);
    tabCell.innerHTML = cellData;
  }
}

// Render the thin triples as a preformatted block:
var pre = document.getElementById("thin-triples");
let pre_text = "";
for (const key in thin_rows.by_stanza) {
  thin_rows.by_stanza[key].forEach(function(row) {
    pre_text = pre_text + row.join(' | ') + "\n";
  });
}
pre.textContent = pre_text;

// Render the thick triples as a table:
var divContainer = document.getElementById("thick-triples");
divContainer.innerHTML = "";
divContainer.appendChild(table);
