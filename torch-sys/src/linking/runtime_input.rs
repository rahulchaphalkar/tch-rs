// use once_cell::sync::Lazy;
// use libloading;
// use std::path::PathBuf;
// use std::sync::Arc;
// use std::sync::RwLock;

#[doc(hidden)]
#[macro_export]
macro_rules! link {
    (
        $(

        $(
            $(#[$attr:meta])*
            //$(
            pub struct $struct_name:ident {
                $($field_name:ident: $field_type:ty),* $(,)?
            }
        //)*
        )*
        $(
            pub mod $mod_name:ident;
        )*

        //$(
            extern "C" {
                $(
                    //$(#[$fn_attr:meta])*
                    // pub fn $fn_name:ident($($pname:ident: $pty:ty),* $(,)?) $(-> $ret:ty)*;
                    pub fn $fn_name:ident($($pname:ident: $pty:ty),* $(,)?) $(-> $ret:ty)?;
                )+
            }

    )+
    ) => (
        $(
        //$(
        $(
            $(#[$attr])*
            pub struct $struct_name {
                $($field_name: $field_type),*
            }
        )*

        $(
            pub mod $mod_name;
        )*
        )+

        use once_cell::sync::Lazy;
        use ::libloading;
        use std::path::PathBuf;
        use std::sync::Arc;
        use std::sync::RwLock;

       

        // Wrap the loaded functions.
        #[derive(Debug)]
        pub(crate) struct SharedLibrary {
            library: libloading::Library,
            path: PathBuf,
            pub functions: Functions,
        }
        impl SharedLibrary {
            fn new(library: libloading::Library, path: PathBuf) -> Self {
                Self {
                    library,
                    path,
                    functions: Functions::default(),
                }
            }
        }

        // `LIBRARY` holds the shared library reference.
       static LIBRARY: Lazy<RwLock<Option<Arc<SharedLibrary>>>> = Lazy::new(|| RwLock::new(None));
       //static LIBRARIES: Lazy<RwLock<Vec<Arc<SharedLibrary>>>> = Lazy::new(|| RwLock::new(Vec::new()));

        // Helper function for accessing the thread-local version of the library.
        // fn with_library<T, F>(f: F) -> Option<T>
        // where
        //     F: FnOnce(&SharedLibrary) -> T,
        // {
        //     LIBRARY.read().unwrap().as_ref().map(|library| f(&library))
        // }
        fn with_library<T, F>(f: F) -> Option<T>
        where
            // F: FnOnce(&SharedLibrary) -> Option<T>,
            F: Fn(&SharedLibrary) -> Option<T>,
        {
            // let libraries = LIBRARIES.read().unwrap();
            // println!("Number of libraries: {}", libraries.len());
            // for library in LIBRARIES.read().unwrap().iter() {
            //     if let Some(result) = f(&library) {
            //         return Some(result);
            //     }
            // }
            // for (index, library) in libraries.iter().enumerate() {
            //     println!("Checking library at index: {}", index);
            //     let result = f(&library);
            //     if let Some(_) = &result {
            //         println!("Found a result in library at index: {}", index);
            //         return result;
            //     } else {
            //         println!("No result in library at index: {}", index);
            //     }
            // }
            LIBRARY.read().unwrap().as_ref().map(|library| f(&library)).expect("No library found")
            //println!("nope");
            //None
        }

        //The set of functions loaded dynamically.
        #[derive(Debug, Default)]
        pub(crate) struct Functions {
            $(
            $(
                pub $fn_name: Option<unsafe extern fn($($pname: $pty),*) $(-> $ret)*>,
            )+
        )*
        }

        // Provide functions to load each name from the shared library into the `SharedLibrary`
        // struct.
        mod load {
            $(
                $(
                pub(crate) fn $fn_name(library: &mut super::SharedLibrary) {
                    let symbol = unsafe { library.library.get(stringify!($name).as_bytes()) }.ok();
                    library.functions.$fn_name = match symbol {
                        Some(s) => *s,
                        None => None,
                    };
                }
            )+
        )+
        }

         /// Load all of the function definitions from a shared library.
        ///
        /// # Errors
        ///
        /// May fail if the `openvino-finder` cannot discover the library on the current system.
        pub fn load() -> Result<(), String> {
            //match $crate::library::find() {
            // match PathBuf::from("/home/rahul/repos/pytorch-bindings/pytorch/pytorch-install/lib") {
            //     None => Err("Unable to find the `openvino_c` library to load".into()),
            //     Some(path) => load_from(path),
            // }
            // let path = PathBuf::from("/home/rahul/repos/pytorch-bindings/pytorch/pytorch-install/lib/libtorch_cpu.so");
            let path = PathBuf::from("/home/rahul/repos/pytorch_bindings/libtorch/lib/libtorch_cpu.so");
            println!("Attempting to load shared libraries");
            // let paths = vec![
            //     PathBuf::from("/home/rahul/repos/pytorch-bindings/pytorch/pytorch-install/lib/libtorch.so"),
            //     PathBuf::from("/home/rahul/repos/pytorch-bindings/pytorch/pytorch-install/lib/libtorch_cpu.so"),
            //     PathBuf::from("/home/rahul/repos/pytorch-bindings/pytorch/pytorch-install/lib/libc10.so"),
            // ];
            load_from(path)
            // for path in paths {
            //     load_from(path)?;
            // }
            //Ok(())
        }
        fn load_from(path: PathBuf) -> Result<(), String> {
            println!("Attempting to load shared library from path: {}", path.display());
            let library = Arc::new(SharedLibrary::load(path)?);
            *LIBRARY.write().unwrap() = Some(library);
            //LIBRARIES.write().unwrap().push(library);
            Ok(())
        }

        impl SharedLibrary {
            fn load(path: PathBuf) -> Result<SharedLibrary, String> {
                unsafe {
                    // let library = libloading::Library::new(&path).map_err(|e| {
                    //     format!(
                    //         "the shared library at {} could not be opened: {}",
                    //         path.display(),
                    //         e,
                    //     )
                    // });
                    let library = libloading::Library::new(&path).expect("the shared library could not be opened");
                    //println!("Attempting to load shared library from path: {}", path.display());
                    let mut library = SharedLibrary::new(library, path);
                    $($(load::$fn_name(&mut library);)+)+
                    Ok(library)
                }
            }
        }

        // For each loaded function, we redefine them to proxy their call through the SharedLibrary
        // on the local thread and into the loaded shared library implementation.

        // $($(
        //     pub unsafe fn $fn_name($($pname: $pty), *) $(-> $ret)* {
        //         let f = with_library(|l| {
        //             l.functions.$fn_name.expect(concat!(
        //                 "`libtorch_cpu.so` function not loaded: `",
        //                 stringify!($fn_name)
        //             ))
        //         }).expect("a `libtorch_cpu.so` shared library is not loaded on this thread");
        //         f($($pname), *)
        //     }
        // )+)+
        
        $($(
            pub unsafe fn $fn_name($($pname: $pty), *) $(-> $ret)* {
                let f = with_library(|l| {
                    l.functions.$fn_name/*.as_ref().expect(concat!(
                        "`libtorch_cpu.so` function not loaded: `",
                        stringify!($fn_name)
                    ))*/
                }).expect("a `libtorch_cpu.so` shared library is not loaded on this thread");
                f($($pname), *)
            }
        )+)+

    )
}
