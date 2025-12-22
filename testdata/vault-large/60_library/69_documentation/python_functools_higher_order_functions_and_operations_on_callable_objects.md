---
title: python_functools_higher_order_functions_and_operations_on_callable_objects
uuid:
aliases:
  - "`functools`: Higher-order Functions and Operations on Callable Objects"
  - "`functools` — Higher-order Functions and Operations on Callable Objects"
  - "Python functools: Higher-order Functions and Operations on Callable Objects"
  - "functools: higher-order functions and operations on callable objects"
  - functools_higher_order_functions_and_operations_on_callable_objects
  - python_functools_higher_order_functions_and_operations_on_callable_objects
main_title: "functools"
subtitle: "Higher-order functions and operations on callable objects"
author:
editor:
date_published:
publisher:
  - "[[python_software_foundation|Python Software Foundation]]"
url: "https://docs.python.org/3/library/functools.html"
about: |-
  The `functools` module offers a collection of tools that simplify working with functions and callable objects. It includes utilities to modify, extend or optimize functions without rewriting their core logic, helping you write cleaner and more efficient code.
status: resource
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2025-10-06T17:26
date_modified: 2025-10-08T13:15
tags:
---
# `functools`: Higher-order Functions and Operations on Callable Objects

> [!documentation] Documentation Details
>
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Link**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.title)`
> - **About**: `dv: this.file.frontmatter.about`

---

## [[python|Python]] `functools` — Higher-order Functions and Operations on Callable Objects

**Source code:**[Lib/functools.py](https://github.com/python/cpython/tree/3.13/Lib/functools.py)

---

The module is for higher-order functions: functions that act on or return other functions. In general, any callable object can be treated as a function for the purposes of this module.

The module defines the following functions:

### @functools.cache (*`user_function`*) [¶](https://docs.python.org/3/library/#functools.cache)

Simple lightweight unbounded function cache. Sometimes called [“memoize”](https://en.wikipedia.org/wiki/Memoization).

Returns the same as `lru_cache(maxsize=None)`, creating a thin wrapper around a dictionary lookup for the function arguments. Because it never needs to evict old values, this is smaller and faster than with a size limit.

For example:

```python
@cache
def factorial(n):
    return n * factorial(n-1) if n else 1

>>> factorial(10)      # no previously cached result, makes 11 recursive calls
3628800
>>> factorial(5)       # just looks up cached value result
120
>>> factorial(12)      # makes two new recursive calls, the other 10 are cached
479001600
```

The cache is threadsafe so that the wrapped function can be used in multiple threads. This means that the underlying data structure will remain coherent during concurrent updates.

It is possible for the wrapped function to be called more than once if another thread makes an additional call before the initial call has been completed and cached.

### @functools.cached\_property (*func*) [¶](https://docs.python.org/3/library/#functools.cached_property)

Transform a method of a class into a property whose value is computed once and then cached as a normal attribute for the life of the instance. Similar to [`property()`](https://docs.python.org/3/library/functions.html#property "property"), with the addition of caching. Useful for expensive computed properties of instances that are otherwise effectively immutable.

Example:

```python
class DataSet:

    def __init__(self, sequence_of_numbers):
        self._data = tuple(sequence_of_numbers)

    @cached_property
    def stdev(self):
        return statistics.stdev(self._data)
```

The mechanics of are somewhat different from [`property()`](https://docs.python.org/3/library/functions.html#property "property"). A regular property blocks attribute writes unless a setter is defined. In contrast, a *cached\_property* allows writes.

The *cached\_property* decorator only runs on lookups and only when an attribute of the same name doesn't exist. When it does run, the *cached\_property* writes to the attribute with the same name. Subsequent attribute reads and writes take precedence over the *cached\_property* method and it works like a normal attribute.

The cached value can be cleared by deleting the attribute. This allows the *cached\_property* method to run again.

The *cached\_property* does not prevent a possible race condition in multi-threaded usage. The getter function could run more than once on the same instance, with the latest run setting the cached value. If the cached property is idempotent or otherwise not harmful to run more than once on an instance, this is fine. If synchronization is needed, implement the necessary locking inside the decorated getter function or around the cached property access.

Note, this decorator interferes with the operation of [**PEP 412**](https://peps.python.org/pep-0412/) key-sharing dictionaries. This means that instance dictionaries can take more space than usual.

Also, this decorator requires that the `__dict__` attribute on each instance be a mutable mapping. This means it will not work with some types, such as metaclasses (since the `__dict__` attributes on type instances are read-only proxies for the class namespace), and those that specify `__slots__` without including `__dict__` as one of the defined slots (as such classes don't provide a `__dict__` attribute at all).

If a mutable mapping is not available or if space-efficient key sharing is desired, an effect similar to can also be achieved by stacking [`property()`](https://docs.python.org/3/library/functions.html#property "property") on top of. See [How do I cache method calls?](https://docs.python.org/3/faq/programming.html#faq-cache-method-calls) for more details on how this differs from.

Changed in version 3.12: Prior to Python 3.12, `cached_property` included an undocumented lock to ensure that in multi-threaded usage the getter function was guaranteed to run only once per instance. However, the lock was per-property, not per-instance, which could result in unacceptably high lock contention. In Python 3.12+ this locking is removed.

### functools.cmp\_to\_key (*func*) [¶](https://docs.python.org/3/library/#functools.cmp_to_key)

Transform an old-style comparison function to a [key function](https://docs.python.org/3/glossary.html#term-key-function). Used with tools that accept key functions (such as [`sorted()`](https://docs.python.org/3/library/functions.html#sorted "sorted"), [`min()`](https://docs.python.org/3/library/functions.html#min "min"),[`max()`](https://docs.python.org/3/library/functions.html#max "max"), [`heapq.nlargest()`](https://docs.python.org/3/library/heapq.html#heapq.nlargest "heapq.nlargest"), [`heapq.nsmallest()`](https://docs.python.org/3/library/heapq.html#heapq.nsmallest "heapq.nsmallest"),[`itertools.groupby()`](https://docs.python.org/3/library/itertools.html#itertools.groupby "itertools.groupby")). This function is primarily used as a transition tool for programs being converted from Python 2 which supported the use of comparison functions.

A comparison function is any callable that accepts two arguments, compares them, and returns a negative number for less-than, zero for equality, or a positive number for greater-than. A key function is a callable that accepts one argument and returns another value to be used as the sort key.

Example:

```python
sorted(iterable, key=cmp_to_key(locale.strcoll))  # locale-aware sort order
```

For sorting examples and a brief sorting tutorial, see [Sorting Techniques](https://docs.python.org/3/howto/sorting.html#sortinghowto).

### @ functools.lru\_cache (*`user_function`*) [¶](https://docs.python.org/3/library/#functools.lru_cache)

@ functools.lru\_cache (*maxsize = 128*, *typed = False*)

Decorator to wrap a function with a memoizing callable that saves up to the *`maxsize`* most recent calls. It can save time when an expensive or I/O bound function is periodically called with the same arguments.

The cache is threadsafe so that the wrapped function can be used in multiple threads. This means that the underlying data structure will remain coherent during concurrent updates.

It is possible for the wrapped function to be called more than once if another thread makes an additional call before the initial call has been completed and cached.

Since a dictionary is used to cache results, the positional and keyword arguments to the function must be [hashable](https://docs.python.org/3/glossary.html#term-hashable).

Distinct argument patterns may be considered to be distinct calls with separate cache entries. For example, `f(a=1, b=2)` and `f(b=2, a=1)` differ in their keyword argument order and may have two separate cache entries.

If *`user_function`* is specified, it must be a callable. This allows the *`lru_cache`* decorator to be applied directly to a user function, leaving the *`maxsize`* at its default value of 128:

```python
@lru_cache
def count_vowels(sentence):
    return sum(sentence.count(vowel) for vowel in 'AEIOUaeiou')
```

If *`maxsize`* is set to `None`, the LRU feature is disabled and the cache can grow without bound.

If *`typed`* is set to true, function arguments of different types will be cached separately. If *`typed`* is false, the implementation will usually regard them as equivalent calls and only cache a single result. (Some types such as *str* and *int* may be cached separately even when *`typed`* is false.)

Note, type specificity applies only to the function's immediate arguments rather than their contents. The scalar arguments, `Decimal(42)` and `Fraction(42)` are be treated as distinct calls with distinct results. In contrast, the tuple arguments `('answer', Decimal(42))` and `('answer', Fraction(42))` are treated as equivalent.

The wrapped function is instrumented with a `cache_parameters()` function that returns a new [`dict`](https://docs.python.org/3/library/stdtypes.html#dict "dict") showing the values for *`maxsize`* and *`typed`*. This is for information purposes only. Mutating the values has no effect.

To help measure the effectiveness of the cache and tune the *`maxsize`* parameter, the wrapped function is instrumented with a `cache_info()` function that returns a [named tuple](https://docs.python.org/3/glossary.html#term-named-tuple) showing *hits*, *misses*,*`maxsize`* and *currsize*.

The decorator also provides a `cache_clear()` function for clearing or invalidating the cache.

The original underlying function is accessible through the `__wrapped__` attribute. This is useful for introspection, for bypassing the cache, or for rewrapping the function with a different cache.

The cache keeps references to the arguments and return values until they age out of the cache or until the cache is cleared.

If a method is cached, the `self` instance argument is included in the cache. See [How do I cache method calls?](https://docs.python.org/3/faq/programming.html#faq-cache-method-calls)

An [LRU (least recently used) cache](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_Recently_Used_\(LRU\)) works best when the most recent calls are the best predictors of upcoming calls (for example, the most popular articles on a news server tend to change each day). The cache's size limit assures that the cache does not grow without bound on long-running processes such as web servers.

In general, the LRU cache should only be used when you want to reuse previously computed values. Accordingly, it doesn't make sense to cache functions with side-effects, functions that need to create distinct mutable objects on each call (such as generators and async functions), or impure functions such as time() or random().

Example of an LRU cache for static web content:

```python
@lru_cache(maxsize=32)
def get_pep(num):
    'Retrieve text of a Python Enhancement Proposal'
    resource = f'https://peps.python.org/pep-{num:04d}'
    try:
        with urllib.request.urlopen(resource) as s:
            return s.read()
    except urllib.error.HTTPError:
        return 'Not Found'

>>> for n in 8, 290, 308, 320, 8, 218, 320, 279, 289, 320, 9991:
...     pep = get_pep(n)
...     print(n, len(pep))

>>> get_pep.cache_info()
CacheInfo(hits=3, misses=8, maxsize=32, currsize=8)
```

Example of efficiently computing [Fibonacci numbers](https://en.wikipedia.org/wiki/Fibonacci_number) using a cache to implement a [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming) technique:

```python
@lru_cache(maxsize=None)
def fib(n):
    if n < 2:
        return n
    return fib(n-1) + fib(n-2)

>>> [fib(n) for n in range(16)]
[0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610]

>>> fib.cache_info()
CacheInfo(hits=28, misses=16, maxsize=None, currsize=16)
```

Changed in version 3.3: Added the *`typed`* option.

Changed in version 3.8: Added the *`user_function`* option.

Changed in version 3.9: Added the function `cache_parameters()`

### @functools.total\_ordering [¶](https://docs.python.org/3/library/#functools.total_ordering)

Given a class defining one or more rich comparison ordering methods, this class decorator supplies the rest. This simplifies the effort involved in specifying all of the possible rich comparison operations:

The class must define one of [`__lt__()`](https://docs.python.org/3/reference/datamodel.html#object.__lt__ "object.__lt__"), [`__le__()`](https://docs.python.org/3/reference/datamodel.html#object.__le__ "object.__le__"),[`__gt__()`](https://docs.python.org/3/reference/datamodel.html#object.__gt__ "object.__gt__"), or [`__ge__()`](https://docs.python.org/3/reference/datamodel.html#object.__ge__ "object.__ge__"). In addition, the class should supply an [`__eq__()`](https://docs.python.org/3/reference/datamodel.html#object.__eq__ "object.__eq__") method.

For example:

```python
@total_ordering
class Student:
    def _is_valid_operand(self, other):
        return (hasattr(other, "lastname") and
                hasattr(other, "firstname"))
    def __eq__(self, other):
        if not self._is_valid_operand(other):
            return NotImplemented
        return ((self.lastname.lower(), self.firstname.lower()) ==
                (other.lastname.lower(), other.firstname.lower()))
    def __lt__(self, other):
        if not self._is_valid_operand(other):
            return NotImplemented
        return ((self.lastname.lower(), self.firstname.lower()) <
                (other.lastname.lower(), other.firstname.lower()))
```

> [!Note]
>
> While this decorator makes it easy to create well behaved totally ordered types, it *does* come at the cost of slower execution and more complex stack traces for the derived comparison methods. If performance benchmarking indicates this is a bottleneck for a given application, implementing all six rich comparison methods instead is likely to provide an easy speed boost.

> [!Note]
>
> This decorator makes no attempt to override methods that have been declared in the class *or its superclasses*. Meaning that if a superclass defines a comparison operator, *`total_ordering`* will not implement it again, even if the original method is abstract.

Changed in version 3.4: Returning `NotImplemented` from the underlying comparison function for unrecognised types is now supported.

### functools.partial (*func*, */*, *\* args*, *\*\* keywords*) [¶](https://docs.python.org/3/library/#functools.partial)

Return a new which when called will behave like *func* called with the positional arguments *args* and keyword arguments *keywords*. If more arguments are supplied to the call, they are appended to *args*. If additional keyword arguments are supplied, they extend and override *keywords*. Roughly equivalent to:

```python
def partial(func, /, *args, **keywords):
    def newfunc(*fargs, **fkeywords):
        newkeywords = {**keywords, **fkeywords}
        return func(*args, *fargs, **newkeywords)
    newfunc.func = func
    newfunc.args = args
    newfunc.keywords = keywords
    return newfunc
```

The is used for partial function application which "freezes" some portion of a function's arguments and/or keywords resulting in a new object with a simplified signature. For example, can be used to create a callable that behaves like the [`int()`](https://docs.python.org/3/library/functions.html#int "int") function where the *base* argument defaults to two:

```python
>>> from functools import partial
>>> basetwo = partial(int, base=2)
>>> basetwo.__doc__ = 'Convert base 2 string to an int.'
>>> basetwo('10010')
18
```

### *class* functools.partialmethod (*func*, */*, *\* args*, *\*\* keywords*) [¶](https://docs.python.org/3/library/#functools.partialmethod)

Return a new descriptor which behaves like except that it is designed to be used as a method definition rather than being directly callable.

*func* must be a [descriptor](https://docs.python.org/3/glossary.html#term-descriptor) or a callable (objects which are both, like normal functions, are handled as descriptors).

When *func* is a descriptor (such as a normal Python function,[`classmethod()`](https://docs.python.org/3/library/functions.html#classmethod "classmethod"), [`staticmethod()`](https://docs.python.org/3/library/functions.html#staticmethod "staticmethod"), [`abstractmethod()`](https://docs.python.org/3/library/abc.html#abc.abstractmethod "abc.abstractmethod") or another instance of), calls to `__get__` are delegated to the underlying descriptor, and an appropriate returned as the result.

When *func* is a non-descriptor callable, an appropriate bound method is created dynamically. This behaves like a normal Python function when used as a method: the *self* argument will be inserted as the first positional argument, even before the *args* and *keywords* supplied to the constructor.

Example:

```python
>>> class Cell:
...     def __init__(self):
...         self._alive = False
...     @property
...     def alive(self):
...         return self._alive
...     def set_state(self, state):
...         self._alive = bool(state)
...     set_alive = partialmethod(set_state, True)
...     set_dead = partialmethod(set_state, False)
...
>>> c = Cell()
>>> c.alive
False
>>> c.set_alive()
>>> c.alive
True
```

### functools.reduce (*function*, *iterable*, \[*initial*, \] */*) [¶](https://docs.python.org/3/library/#functools.reduce)

Apply *function* of two arguments cumulatively to the items of *iterable*, from left to right, so as to reduce the iterable to a single value. For example,`reduce(lambda x, y: x+y, [1, 2, 3, 4, 5])` calculates `((((1+2)+3)+4)+5)`. The left argument, *x*, is the accumulated value and the right argument, *y*, is the update value from the *iterable*. If the optional *initial* is present, it is placed before the items of the iterable in the calculation, and serves as a default when the iterable is empty. If *initial* is not given and *iterable* contains only one item, the first item is returned.

Roughly equivalent to:

```python
initial_missing = object()

def reduce(function, iterable, initial=initial_missing, /):
    it = iter(iterable)
    if initial is initial_missing:
        value = next(it)
    else:
        value = initial
    for element in it:
        value = function(value, element)
    return value
```

See [`itertools.accumulate()`](https://docs.python.org/3/library/itertools.html#itertools.accumulate "itertools.accumulate") for an iterator that yields all intermediate values.

### @functools.singledispatch [¶](https://docs.python.org/3/library/#functools.singledispatch)

Transform a function into a [single-dispatch](https://docs.python.org/3/glossary.html#term-single-dispatch) [generic function](https://docs.python.org/3/glossary.html#term-generic-function).

To define a generic function, decorate it with the `@singledispatch` decorator. When defining a function using `@singledispatch`, note that the dispatch happens on the type of the first argument:

```python
>>> from functools import singledispatch
>>> @singledispatch
... def fun(arg, verbose=False):
...     if verbose:
...         print("Let me just say,", end=" ")
...     print(arg)
```

[`types.UnionType`](https://docs.python.org/3/library/types.html#types.UnionType "types.UnionType") and [`typing.Union`](https://docs.python.org/3/library/typing.html#typing.Union "typing.Union") can also be used:

For code which doesn't use type annotations, the appropriate type argument can be passed explicitly to the decorator itself:

For code that dispatches on a collections type (e.g., `list`), but wants to typehint the items of the collection (e.g., `list[int]`), the dispatch type should be passed explicitly to the decorator itself with the typehint going into the function definition:

> [!Note]
>
> At runtime the function will dispatch on an instance of a list regardless of the type contained within the list i.e. `[1,2,3]` will be dispatched the same as `["foo", "bar", "baz"]`. The annotation provided in this example is for static type checkers only and has no runtime impact.

To enable registering [lambdas](https://docs.python.org/3/glossary.html#term-lambda) and pre-existing functions, the attribute can also be used in a functional form:

The attribute returns the undecorated function. This enables decorator stacking, [`pickling`](https://docs.python.org/3/library/pickle.html#module-pickle "pickle: Convert Python objects to streams of bytes and back."), and the creation of unit tests for each variant independently:

When called, the generic function dispatches on the type of the first argument:

```python
>>> fun("Hello, world.")
Hello, world.
>>> fun("test.", verbose=True)
Let me just say, test.
>>> fun(42, verbose=True)
Strength in numbers, eh? 42
>>> fun(['spam', 'spam', 'eggs', 'spam'], verbose=True)
Enumerate this:
0 spam
1 spam
2 eggs
3 spam
>>> fun(None)
Nothing.
>>> fun(1.23)
0.615
```

Where there is no registered implementation for a specific type, its method resolution order is used to find a more generic implementation. The original function decorated with `@singledispatch` is registered for the base [`object`](https://docs.python.org/3/library/functions.html#object "object") type, which means it is used if no better implementation is found.

If an implementation is registered to an [abstract base class](https://docs.python.org/3/glossary.html#term-abstract-base-class), virtual subclasses of the base class will be dispatched to that implementation:

To check which implementation the generic function will choose for a given type, use the `dispatch()` attribute:

```python
>>> fun.dispatch(float)
<function fun_num at 0x1035a2840>
>>> fun.dispatch(dict)    # note: default implementation
<function fun at 0x103fe0000>
```

To access all registered implementations, use the read-only `registry` attribute:

```python
>>> fun.registry.keys()
dict_keys([<class 'NoneType'>, <class 'int'>, <class 'object'>,
          <class 'decimal.Decimal'>, <class 'list'>,
          <class 'float'>])
>>> fun.registry[float]
<function fun_num at 0x1035a2840>
>>> fun.registry[object]
<function fun at 0x103fe0000>
```

### *class* functools.singledispatchmethod (*func*) [¶](https://docs.python.org/3/library/#functools.singledispatchmethod)

Transform a method into a [single-dispatch](https://docs.python.org/3/glossary.html#term-single-dispatch) [generic function](https://docs.python.org/3/glossary.html#term-generic-function).

To define a generic method, decorate it with the `@singledispatchmethod` decorator. When defining a function using `@singledispatchmethod`, note that the dispatch happens on the type of the first non- *self* or non- *cls* argument:

`@singledispatchmethod` supports nesting with other decorators such as [`@classmethod`](https://docs.python.org/3/library/functions.html#classmethod "classmethod"). Note that to allow for `dispatcher.register`, `singledispatchmethod` must be the *outer most* decorator. Here is the `Negator` class with the `neg` methods bound to the class, rather than an instance of the class:

The same pattern can be used for other similar decorators:[`@staticmethod`](https://docs.python.org/3/library/functions.html#staticmethod "staticmethod"), [`@~abc.abstractmethod`](https://docs.python.org/3/library/abc.html#abc.abstractmethod "abc.abstractmethod"), and others.

### functools.update\_wrapper (*wrapper*, *wrapped*, *assigned = WRAPPER\_ASSIGNMENTS*, *updated = WRAPPER\_UPDATES*) [¶](https://docs.python.org/3/library/#functools.update_wrapper)

Update a *wrapper* function to look like the *wrapped* function. The optional arguments are tuples to specify which attributes of the original function are assigned directly to the matching attributes on the wrapper function and which attributes of the wrapper function are updated with the corresponding attributes from the original function. The default values for these arguments are the module level constants `WRAPPER_ASSIGNMENTS` (which assigns to the wrapper function's [`__module__`](https://docs.python.org/3/reference/datamodel.html#function.__module__ "function.__module__"), [`__name__`](https://docs.python.org/3/reference/datamodel.html#function.__name__ "function.__name__"),[`__qualname__`](https://docs.python.org/3/reference/datamodel.html#function.__qualname__ "function.__qualname__"), [`__annotations__`](https://docs.python.org/3/reference/datamodel.html#function.__annotations__ "function.__annotations__"),[`__type_params__`](https://docs.python.org/3/reference/datamodel.html#function.__type_params__ "function.__type_params__"), and [`__doc__`](https://docs.python.org/3/reference/datamodel.html#function.__doc__ "function.__doc__"), the documentation string) and `WRAPPER_UPDATES` (which updates the wrapper function's [`__dict__`](https://docs.python.org/3/reference/datamodel.html#function.__dict__ "function.__dict__"), i.e. the instance dictionary).

To allow access to the original function for introspection and other purposes (e.g. bypassing a caching decorator such as), this function automatically adds a `__wrapped__` attribute to the wrapper that refers to the function being wrapped.

The main intended use for this function is in [decorator](https://docs.python.org/3/glossary.html#term-decorator) functions which wrap the decorated function and return the wrapper. If the wrapper function is not updated, the metadata of the returned function will reflect the wrapper definition rather than the original function definition, which is typically less than helpful.

may be used with callables other than functions. Any attributes named in *assigned* or *updated* that are missing from the object being wrapped are ignored (i.e. this function will not attempt to set them on the wrapper function). [`AttributeError`](https://docs.python.org/3/library/exceptions.html#AttributeError "AttributeError") is still raised if the wrapper function itself is missing any attributes named in *updated*.

Changed in version 3.2: The `__wrapped__` attribute is now automatically added. The [`__annotations__`](https://docs.python.org/3/reference/datamodel.html#function.__annotations__ "function.__annotations__") attribute is now copied by default. Missing attributes no longer trigger an [`AttributeError`](https://docs.python.org/3/library/exceptions.html#AttributeError "AttributeError").

Changed in version 3.4: The `__wrapped__` attribute now always refers to the wrapped function, even if that function defined a `__wrapped__` attribute. (see [bpo-17482](https://bugs.python.org/issue?@action=redirect&bpo=17482))

Changed in version 3.12: The [`__type_params__`](https://docs.python.org/3/reference/datamodel.html#function.__type_params__ "function.__type_params__") attribute is now copied by default.

### @functools.wraps (*wrapped*, *assigned = WRAPPER\_ASSIGNMENTS*, *updated = WRAPPER\_UPDATES*) [¶](https://docs.python.org/3/library/#functools.wraps)

This is a convenience function for invoking as a function decorator when defining a wrapper function. It is equivalent to `partial(update_wrapper, wrapped=wrapped, assigned=assigned, updated=updated)`. For example:

```python
>>> from functools import wraps
>>> def my_decorator(f):
...     @wraps(f)
...     def wrapper(*args, **kwds):
...         print('Calling decorated function')
...         return f(*args, **kwds)
...     return wrapper
...
>>> @my_decorator
... def example():
...     """Docstring"""
...     print('Called example function')
...
>>> example()
Calling decorated function
Called example function
>>> example.__name__
'example'
>>> example.__doc__
'Docstring'
```

Without the use of this decorator factory, the name of the example function would have been `'wrapper'`, and the docstring of the original `example()` would have been lost.

## [`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") Objects

[`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") objects are callable objects created by [`partial()`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial"). They have three read-only attributes:

- [`partial.func`](https://docs.python.org/3/library/functools.html#functools.partial.func)
	- A callable object or function. Calls to the [`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") object will be forwarded to [`func`](https://docs.python.org/3/library/functools.html#functools.partial.func "functools.partial.func") with new arguments and keywords.
- [`partial.args`](https://docs.python.org/3/library/functools.html#functools.partial.args)
	- The leftmost positional arguments that will be prepended to the positional arguments provided to a [`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") object call.
- [`partial.keywords`](https://docs.python.org/3/library/functools.html#functools.partial.keywords)
	- The keyword arguments that will be supplied when the [`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") object is called.

[`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") objects are like [function objects](https://docs.python.org/3/reference/datamodel.html#user-defined-funcs) in that they are callable, weak referenceable, and can have attributes. There are some important differences. For instance, the [`__name__`](https://docs.python.org/3/reference/datamodel.html#function.__name__ "function.__name__") and [`function.__doc__`](https://docs.python.org/3/reference/datamodel.html#function.__doc__ "function.__doc__") attributes are not created automatically. Also, [`partial`](https://docs.python.org/3/library/functools.html#functools.partial "functools.partial") objects defined in classes behave like static methods and do not transform into bound methods during instance attribute look-up.
