use super::slice_types::{MemoryError, SignalSlice, SliceCapacity};
use crate::execution_data::type_definitions::NodePointer;
use crate::execution_data::ExecutedProgram;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct ComponentRepresentation {
    pub node_pointer: Option<NodePointer>,
    unassigned_inputs: HashMap<String, SliceCapacity>,
    inputs: HashMap<String, SignalSlice>,
    outputs: HashMap<String, SignalSlice>,
}

impl ComponentRepresentation {
    pub fn initialize_component(
        component: &mut ComponentRepresentation,
        node_pointer: NodePointer,
        scheme: &ExecutedProgram,
    ) -> Result<(), MemoryError> {
        if component.is_initialized() {
            return Err(MemoryError::AssignmentError);
        }
        let possible_node = ExecutedProgram::get_node(scheme, node_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();

        let mut unassigned_inputs = HashMap::new();
        let mut inputs = HashMap::new();
        for (symbol, route) in node.inputs() {
            let signal_slice = SignalSlice::new_with_route(route, &false);
            let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
            if signal_slice_size > 0 {
                unassigned_inputs.insert(symbol.clone(), signal_slice_size);
            }
            inputs.insert(symbol.clone(), signal_slice);
        }

        let mut outputs = HashMap::new();
        for (symbol, route) in node.outputs() {
            outputs.insert(symbol.clone(), SignalSlice::new_with_route(route, &true));
        }
        *component = ComponentRepresentation {
            node_pointer: Some(node_pointer),
            unassigned_inputs,
            inputs,
            outputs,
        };
        Ok(())
    }
    pub fn signal_has_value(
        component: &ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
    ) -> Result<bool, MemoryError> {
        if component.node_pointer.is_none() {
            return Err(MemoryError::InvalidAccess);
        }
        if component.outputs.contains_key(signal_name) && !component.unassigned_inputs.is_empty() {
            return Err(MemoryError::InvalidAccess);
        }

        let slice = if component.inputs.contains_key(signal_name) {
            component.inputs.get(signal_name).unwrap()
        } else {
            component.outputs.get(signal_name).unwrap()
        };
        let enabled = *SignalSlice::get_reference_to_single_value(slice, access)?;
        Ok(enabled)
    }
    pub fn get_signal(&self, signal_name: &str) -> Result<&SignalSlice, MemoryError> {
        if self.node_pointer.is_none() {
            return Err(MemoryError::InvalidAccess);
        }
        if self.outputs.contains_key(signal_name) && !self.unassigned_inputs.is_empty() {
            return Err(MemoryError::InvalidAccess);
        }

        let slice = if self.inputs.contains_key(signal_name) {
            self.inputs.get(signal_name).unwrap()
        } else {
            self.outputs.get(signal_name).unwrap()
        };
        Ok(slice)
    }

    pub fn assign_value_to_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
    ) -> Result<(), MemoryError> {
        let signal_has_value =
            ComponentRepresentation::signal_has_value(component, signal_name, access)?;
        if signal_has_value {
            return Err(MemoryError::AssignmentError);
        }

        let slice = component.inputs.get_mut(signal_name).unwrap();
        let value = SignalSlice::get_mut_reference_to_single_value(slice, access)?;
        let left = component.unassigned_inputs.get_mut(signal_name).unwrap();
        *left -= 1;
        *value = true;
        if *left == 0 {
            component.unassigned_inputs.remove(signal_name);
        }
        Ok(())
    }
    pub fn is_initialized(&self) -> bool {
        self.node_pointer.is_some()
    }
}
