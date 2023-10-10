use crate::{debug, error::Error, input::Result, types::ResultType};
use core::fmt::{DebugStruct, Formatter};

enum State<'a, 'b: 'a> {
    Start(&'a mut Formatter<'b>),
    Struct(DebugStruct<'a, 'b>),
    Moved,
}

#[repr(transparent)]
struct Print<'a, 'b: 'a> {
    state: State<'a, 'b>,
}

const STRUCT_NAME: &str = "FuncType";

impl<'a, 'b: 'a> Print<'a, 'b> {
    fn new(fmt: &'a mut Formatter<'b>) -> Self {
        Self {
            state: State::Start(fmt),
        }
    }

    fn finish(self) -> DebugStruct<'a, 'b> {
        match self.state {
            State::Start(fmt) => fmt.debug_struct(STRUCT_NAME),
            State::Struct(debug) => debug,
            State::Moved => unreachable!(),
        }
    }

    fn debug_struct(&mut self) -> &mut DebugStruct<'a, 'b> {
        if let State::Start(fmt) = core::mem::replace(&mut self.state, State::Moved) {
            self.state = State::Struct(fmt.debug_struct(STRUCT_NAME));
        }

        if let State::Struct(debug) = &mut self.state {
            debug
        } else {
            unreachable!()
        }
    }
}

impl<'i> crate::types::ParseFuncType<'i, Error<'i>> for Print<'_, '_> {
    fn parameters(&mut self, parameters: &mut ResultType<'i, Error<'i>>) -> Result<(), Error<'i>> {
        // TODO: Use DebugParse with ResultType
        self.debug_struct().field("parameters", parameters);
        Ok(())
    }

    fn results(&mut self, results: &mut ResultType<'i, Error<'i>>) -> Result<(), Error<'i>> {
        self.debug_struct().field("results", results);
        Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct DebugFuncType<'i> {
    input: &'i [u8],
}

impl<'i> DebugFuncType<'i> {
    pub(crate) fn new(input: &'i [u8]) -> Self {
        Self { input }
    }
}

impl<'i> debug::DebugParse<'i> for DebugFuncType<'i> {
    fn format<'a, 'b: 'a>(self, f: &'a mut Formatter<'b>) -> debug::Result<'i, 'a, 'b> {
        let mut print = Print::new(f);
        match crate::types::func_type(self.input, &mut print) {
            Ok((input, _)) => print.finish().finish().map(|()| input).map_err(Into::into),
            Err(error) => Err(debug::ParseFailed::new(print.finish(), error).into()),
        }
    }
}
