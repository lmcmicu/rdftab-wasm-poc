mod utils;

use serde_json::{
    // SerdeMap by default backed by BTreeMap (see https://docs.serde.rs/serde_json/map/index.html)
    Map as SerdeMap,
    Value as SerdeValue,
};
use std::collections::{BTreeMap, BTreeSet};

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[macro_use]
extern crate serde_derive;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Structure that is used to hold thin rows by stanza:
#[derive(Serialize, Deserialize)]
pub struct Thin {
    pub by_stanza: BTreeMap<String, Vec<Vec<String>>>,
}

/// Sends back a serialised version of an empty Thin struct.
#[wasm_bindgen]
pub fn initialise_thin() -> JsValue {
    utils::set_panic_hook();
    let rows = Thin {
        by_stanza: BTreeMap::new(),
    };
    JsValue::from_serde(&rows).unwrap()
}

/// Converts thin rows to thick rows, which are then serialised and returned.
#[wasm_bindgen]
pub fn thin_to_thick(thin: &JsValue) -> JsValue {
    let rows: Thin = thin.into_serde().unwrap();
    let mut thick_rows: Vec<_> = vec![];
    for (_, thin_rows) in rows.by_stanza.iter() {
        let subjects = annotate_reify(thin_rows_to_subjects(&thin_rows));
        thick_rows.append(&mut subjects_to_thick_rows(&subjects));
    }
    JsValue::from_serde(&thick_rows).unwrap()
}

//////////////////////////////////////////////////////////////////////////////////////
// The code below has been copied from
// https://github.com/ontodev/rdftab.rs/tree/271c36f3670fe1104c1b62da82f5538b2631e0c9
//
// The rdftab.rs code is copied unchanged, with the single exception of the
// get_cell_contents() function. In rdftab.rs, this accepts an Option<String>
// parameter, which is needed because thin rows are being parsed from a file. Here,
// the thin row fields are already of datatype String (instead of Option<String>).
/////////////////////////////////////////////////////////////////////////////////////

/// Simply return a copy of the given string. Obviously this isn't needed, but it is kept here
/// because by having this function, the rest of the code can remain identical to the rdftab code.
fn get_cell_contents(c: &String) -> String {
    c.to_string()
}

/// Convert the given row to a SerdeValue::Object
fn row2object_map(row: &Vec<String>) -> SerdeValue {
    let object = get_cell_contents(&row[3]);
    let value = get_cell_contents(&row[4]);
    let datatype = get_cell_contents(&row[5]);
    let language = get_cell_contents(&row[6]);

    let mut object_map = SerdeMap::new();
    if object != "" {
        object_map.insert(String::from("object"), SerdeValue::String(object));
    }
    else {
        object_map.insert(String::from("value"), SerdeValue::String(value));
        if datatype != "" {
            object_map.insert(String::from("datatype"), SerdeValue::String(datatype));
        }
        else if language != "" {
            object_map.insert(String::from("language"), SerdeValue::String(language));
        }
    }

    return SerdeValue::Object(object_map);
}

/// Given a SerdeMap mapping strings to SerdeValues, and a specific predicate represented by a
/// string slice, return a SerdeValue representing the first object contained in the predicates map.
fn first_object(predicates: &SerdeMap<String, SerdeValue>, predicate: &str) -> SerdeValue {
    match predicates.get(predicate) {
        None => (),
        Some(objs) => match objs {
            SerdeValue::Array(v) => {
                for obj in v.iter() {
                    if let Some(o) = obj.get("object") {
                        return o.clone();
                    }
                    else if let Some(o) = obj.get("value") {
                        return o.clone();
                    }
                }
            }
            _ => (),
        },
    };
    eprintln!("WARNING No object found");
    return SerdeValue::String(String::from(""));
}

/// Given a subject id, a map representing subjects, a map that compressed versions of the subjects
/// map will be copied to, a set of subject ids to be marked for removal, and the subject,
/// predicate, and object types to be compressed, write a compressed version of subjects to
/// compressed_subjects, and add the eliminated subject ids to the list of those marked for removal.
fn compress(
    kind: &str, subject_id: &String, subjects: &SerdeMap<String, SerdeValue>,
    compressed_subjects: &mut SerdeMap<String, SerdeValue>, remove: &mut BTreeSet<String>,
    preds: &SerdeMap<String, SerdeValue>, subject_type: &str, predicate_type: &str,
    object_type: &str,
) {
    let subject = format!("{}", first_object(&preds, subject_type))
        .trim_start_matches("\"")
        .trim_end_matches("\"")
        .to_string();
    let predicate = format!("{}", first_object(&preds, predicate_type))
        .trim_start_matches("\"")
        .trim_end_matches("\"")
        .to_string();
    let obj = format!("{}", first_object(&preds, object_type))
        .trim_start_matches("\"")
        .trim_end_matches("\"")
        .to_string();

    if let Some(SerdeValue::Object(m)) = compressed_subjects.get_mut(subject_id) {
        m.remove(subject_type);
        m.remove(predicate_type);
        m.remove(object_type);
        m.remove("rdf:type");
    }

    let alt_preds: SerdeMap<String, SerdeValue>;
    match subjects.get(&subject) {
        Some(SerdeValue::Object(m)) => alt_preds = m.clone(),
        _ => alt_preds = SerdeMap::new(),
    };
    if let None = compressed_subjects.get(&subject) {
        compressed_subjects.insert(subject.to_string(), SerdeValue::Object(alt_preds.clone()));
    }
    // We are assured compressed_preds will not be None because of the code immediately above, so
    // we simply call unwrap() here:
    let compressed_preds = compressed_subjects.get_mut(&subject).unwrap();
    if let None = compressed_preds.get(&predicate) {
        let compressed_objs: SerdeValue;
        match alt_preds.get(&predicate) {
            Some(SerdeValue::Object(p)) => compressed_objs = SerdeValue::Object(p.clone()),
            _ => compressed_objs = SerdeValue::Object(SerdeMap::new()),
        };
        if let SerdeValue::Object(m) = compressed_preds {
            m.insert(predicate.to_string(), compressed_objs);
        }
    }

    if let Some(SerdeValue::Array(objs)) = compressed_subjects
        .get(&subject)
        .and_then(|preds| preds.get(&predicate))
    {
        let mut objs_copy = vec![];
        for o in objs {
            let mut o = o.clone();
            let o_obj: String;
            let o_val: String;
            let trim = |s: String| {
                format!("{}", s)
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_string()
            };
            match o.get("object") {
                Some(s) => o_obj = trim(format!("{}", s)),
                None => o_obj = String::from(""),
            };
            match o.get("value") {
                Some(s) => o_val = trim(format!("{}", s)),
                None => o_val = String::from(""),
            };

            if o_obj == obj || o_val == obj {
                if let Some(SerdeValue::Object(items)) = compressed_subjects.get(subject_id) {
                    let mut annotations;
                    match o.get(kind) {
                        Some(SerdeValue::Object(m)) => annotations = m.clone(),
                        _ => annotations = SerdeMap::new(),
                    };
                    for (key, val) in items.iter() {
                        let mut annotations_for_key;
                        match annotations.get(key) {
                            Some(SerdeValue::Array(v)) => annotations_for_key = v.clone(),
                            _ => annotations_for_key = vec![],
                        };
                        if let SerdeValue::Array(v) = val {
                            for w in v {
                                annotations_for_key.push(w.clone());
                            }
                        }
                        annotations.insert(key.to_string(), SerdeValue::Array(annotations_for_key));
                    }
                    if let SerdeValue::Object(mut m) = o.clone() {
                        m.insert(kind.to_string(), SerdeValue::Object(annotations));
                        o = SerdeValue::Object(m);
                        remove.insert(subject_id.to_string());
                    }
                    else {
                        eprintln!("WARNING: {} is not a map.", o);
                    }
                }
            }
            objs_copy.push(o);
        }

        if let Some(SerdeValue::Object(m)) = compressed_subjects.get_mut(&subject) {
            if let Some(SerdeValue::Array(v)) = m.get_mut(&predicate) {
                *v = objs_copy;
            }
        }
    }
}

/// Given a vector of thin rows, return a map from Strings to SerdeValues
fn thin_rows_to_subjects(thin_rows: &Vec<Vec<String>>) -> SerdeMap<String, SerdeValue> {
    let mut subjects = SerdeMap::new();
    let mut dependencies: BTreeMap<String, BTreeSet<_>> = BTreeMap::new();
    let mut subject_ids: BTreeSet<String> = vec![].into_iter().collect();
    for row in thin_rows.iter() {
        subject_ids.insert(get_cell_contents(&row[1]));
    }

    for subject_id in &subject_ids {
        let mut predicates = SerdeMap::new();
        for row in thin_rows.iter() {
            if subject_id.ne(&get_cell_contents(&row[1])) {
                continue;
            }

            let object_map = row2object_map(&row);
            // Useful closure for adding SerdeValues to a list in sorted order:
            let add_objects_and_sort = |v: &mut SerdeValue| {
                if let SerdeValue::Array(v) = v {
                    v.push(object_map);
                    v.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                }
            };

            let predicate = get_cell_contents(&row[2]);
            // If the given predicate is already associated with a list in the predicates map,
            // then add the objects represented by `row` to the list in sorted order, otherwise
            // add an empty list corresponding to the predicate in the map.
            if let Some(v) = predicates.get_mut(&predicate) {
                add_objects_and_sort(v);
            }
            else if predicate != "" {
                let mut v = SerdeValue::Array(vec![]);
                add_objects_and_sort(&mut v);
                predicates.insert(predicate, v);
            }
            else {
                eprintln!("WARNING row {:?} has empty predicate", row);
            }

            let object = get_cell_contents(&row[3]);
            // If the object is a blank node, then if a set corresponding to `subject_id` already
            // exists in the dependencies map, add the object to it; otherwise add an empty list
            // corresponding to the subject in the map.
            if object != "" && object.starts_with("_:") {
                if let Some(v) = dependencies.get_mut(subject_id) {
                    v.insert(object);
                }
                else {
                    let mut v = BTreeSet::new();
                    v.insert(object);
                    dependencies.insert(subject_id.to_owned(), v);
                }
            }
        }

        // Add an entry mapping `subject_id` to the predicates map in the subjects map:
        subjects.insert(subject_id.to_owned(), SerdeValue::Object(predicates));
    }

    work_through_dependencies(&mut dependencies, &mut subjects);
    subjects
}

fn work_through_dependencies(
    dependencies: &mut BTreeMap<String, BTreeSet<String>>,
    subjects: &mut SerdeMap<String, SerdeValue>,
) {
    // Work through dependencies from leaves to root, nesting the blank structures:
    while !dependencies.is_empty() {
        let mut leaves: BTreeSet<_> = vec![].into_iter().collect();
        for leaf in subjects.keys() {
            if !dependencies.keys().collect::<Vec<_>>().contains(&leaf) {
                leaves.insert(leaf.to_owned());
            }
        }

        dependencies.clear();
        let mut handled = BTreeSet::new();
        for subject_id in &subjects.keys().map(|s| s.to_owned()).collect::<Vec<_>>() {
            let mut predicates: SerdeMap<String, SerdeValue>;
            match subjects.get(subject_id) {
                Some(SerdeValue::Object(m)) => predicates = m.clone(),
                _ => predicates = SerdeMap::new(),
            };

            for predicate in &predicates.keys().map(|s| s.to_owned()).collect::<Vec<_>>() {
                let pred_objs: Vec<SerdeValue>;
                match predicates.get(predicate) {
                    Some(SerdeValue::Array(v)) => pred_objs = v.clone(),
                    _ => pred_objs = vec![],
                };

                let mut objects = vec![];
                for obj in &pred_objs {
                    let mut obj = obj.to_owned();
                    let o: SerdeValue;
                    if let Some(val) = obj.get(&String::from("object")) {
                        o = val.to_owned();
                    }
                    else {
                        o = SerdeValue::Object(SerdeMap::new());
                    }

                    match o {
                        SerdeValue::String(o) => {
                            if o.starts_with("_:") {
                                if leaves.contains(&o) {
                                    let val: SerdeValue;
                                    if let Some(v) = subjects.get(&o) {
                                        val = v.to_owned();
                                    }
                                    else {
                                        val = SerdeValue::Object(SerdeMap::new());
                                    }

                                    if let SerdeValue::Object(ref mut m) = obj {
                                        m.clear();
                                        m.insert(String::from("object"), val);
                                        handled.insert(o);
                                    }
                                }
                                else {
                                    if let Some(v) = dependencies.get_mut(subject_id) {
                                        v.insert(o);
                                    }
                                    else {
                                        let mut v = BTreeSet::new();
                                        v.insert(o);
                                        dependencies.insert(subject_id.to_owned(), v);
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                    objects.push(obj);
                }
                objects.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                predicates.insert(predicate.to_owned(), SerdeValue::Array(objects));
                subjects.insert(
                    subject_id.to_owned(),
                    SerdeValue::Object(predicates.to_owned()),
                );
            }
        }
        for subject_id in &handled {
            subjects.remove(subject_id);
        }
    }
}

fn annotate_reify(subjects: SerdeMap<String, SerdeValue>) -> SerdeMap<String, SerdeValue> {
    // OWL annotation and RDF reification:
    let mut remove: BTreeSet<String> = vec![].into_iter().collect();
    let mut compressed_subjects = SerdeMap::new();
    for subject_id in subjects.keys() {
        let subject_id = subject_id.to_owned();
        let preds: SerdeMap<String, SerdeValue>;
        match subjects.get(&subject_id) {
            Some(SerdeValue::Object(m)) => preds = m.clone(),
            _ => preds = SerdeMap::new(),
        };

        if let None = compressed_subjects.get(&subject_id) {
            compressed_subjects.insert(subject_id.to_owned(), SerdeValue::Object(preds.clone()));
        };

        if preds.contains_key("owl:annotatedSource") {
            compress(
                "annotations",
                &subject_id,
                &subjects,
                &mut compressed_subjects,
                &mut remove,
                &preds,
                "owl:annotatedSource",
                "owl:annotatedProperty",
                "owl:annotatedTarget",
            );
        }

        if preds.contains_key("rdf:subject") {
            compress(
                "metadata",
                &subject_id,
                &subjects,
                &mut compressed_subjects,
                &mut remove,
                &preds,
                "rdf:subject",
                "rdf:predicate",
                "rdf:object",
            );
        }
    }

    // Remove the subject ids from compressed_subjects that we earlier identified for removal:
    for r in &remove {
        compressed_subjects.remove(r);
    }

    compressed_subjects
}

/// Convert the given SerdeMap, mapping Strings to SerdeValues, into a vector of SerdeMaps that map
/// Strings to SerdeValues.
fn subjects_to_thick_rows(
    subjects: &SerdeMap<String, SerdeValue>,
) -> Vec<SerdeMap<String, SerdeValue>> {
    let mut rows = vec![];
    for subject_id in subjects.keys() {
        let predicates: SerdeMap<String, SerdeValue>;
        match subjects.get(subject_id) {
            Some(SerdeValue::Object(p)) => predicates = p.clone(),
            _ => predicates = SerdeMap::new(),
        };

        for predicate in predicates.keys() {
            let objs: Vec<SerdeValue>;
            match predicates.get(predicate) {
                Some(SerdeValue::Array(v)) => objs = v.clone(),
                _ => objs = vec![],
            };

            for obj in objs {
                let mut result: SerdeMap<String, SerdeValue>;
                match obj {
                    SerdeValue::Object(m) => result = m.clone(),
                    _ => result = SerdeMap::new(),
                };
                result.insert(
                    String::from("subject"),
                    SerdeValue::String(subject_id.clone()),
                );
                result.insert(
                    String::from("predicate"),
                    SerdeValue::String(predicate.clone()),
                );
                if let Some(s) = result.get("object") {
                    match s {
                        SerdeValue::String(_) => (),
                        _ => {
                            let s = s.to_string();
                            result.insert(String::from("object"), SerdeValue::String(s));
                        }
                    };
                }
                rows.push(result);
            }
        }
    }
    rows
}
