# Data

The essential role of actors is to pass data through channels from one client to another client by the mean of their respective actor`Read/Write` interfaces .
The data is encapsulated into a tuple structure `Data<U>`:
```rust,no_run,noplayground
pub struct Data<U: UniqueIdentifier>(<U as UniqueIdentifier>::DataType, PhantomData<U>);
```
Each container `Data<U>` is uniquely defined with a type parameter `U`,
the trait bound on `U` means that `U` must implement the `UniqueIdentifier` trait and the actual type of the data 
that is moved around is given by the trait associated type `UniqueIdentifier::DataType`.

As an example, lets define 2 clients `ClientA` and `ClientB` and a double precision vector `Vec<f64>` that must be transferred from `ClientA` to `ClientB`.
To do so, one needs 
 * first to define `U`:
 ```rust,no_run,noplayground
pub enum A2B {}
```
here `U` is an empty enum. `U` can be of any type however empty enums are very efficient in terms of zero-cost abstraction as they entirely vanished after compilation.
 * then to implement the trait `UniqueIdentifier`:
  ```rust,no_run,noplayground
impl UniqueIdentifer for A2B {
    type DataType = Vec<f64>;
}

```
This is where the actual type of the data to be transferred, is defined.

Note that there is a derive macro `UID` that implements the `UniqueIdentifier` trait on any type that the derive attribute is applied to, so we could have written instead:
 ```rust,no_run,noplayground
#[derive(UID)]
#[uid(data="Vec<f64>")]
pub enum A2B {}
```
The derive macro uses `Vec<f64>` as the default type for `DataType`, so an even simpler declaration is
```rust,no_run,noplayground
#[derive(UID)]
pub enum A2B {}
```

After that the `Read` and `Write` traits are implemented:
 * Write
 ```rust,no_run,noplayground
impl Write<A2B> for ClientA {
    fn write(&mut self) -> Option<Arc<Data<A2B>>> { ... }
}
```
 * Read
 ```rust,no_run,noplayground
impl Read<A2B> for ClientB {
    fn read(&mut self, data: Arc<Data<A2B>>) { ... }
}
```

One may choose as well, to implement the trait `Size<U: UniqueIdentifer>` for some of the clients.
The trait provides the definition of the interface to get the size of the data that is written out:
 ```rust,no_run,noplayground
impl Size<A2B> for ClientA {
    fn len(&self) -> usize {
        get_size_from_client(&self)
    }
}
```

If needs be, an existing type data identifier `U` can be replicated as long as the duplicate applies to the same client.
As an example let define `A2BDPLGR`, the doppelganger of `A2B`:
 ```rust,no_run,noplayground
#[derive(UID)]
#[alias(name = "A2B", client = "ClientA", traits = "Write,Size")]
pub enum A2BDPLGR {}
```
The derive attribute macro in that case will also implements, in addition to the trait `UniqueIdentifier`,
 the traits `Write<A2BDPLGR>` and `Size<A2BDPLGR>` for `ClientA`,
each one being a wrapper for the calls to the implementation of the traits `Write<A2B>` and `Size<A2B>`, respectively.