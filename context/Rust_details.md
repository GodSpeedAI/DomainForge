## Rust foundations for cross-language software

Several projects and foundational libraries can be used as reference implementations or "prior art" when building foundational software in Rust and shipping it to users of other languages like Python, TypeScript, and Ruby.

### Projects Serving as Prior Art

A significant amount of prior art exists in the ecosystem that developers can consult to look up existing patterns and determine what solutions are being used effectively in production codebases.

Reference projects mentioned include:

- Prisma TypeScript ORM: This project utilizes the approach of bridging Rust and other languages, although it has faced challenges related to serializing data across the boundary.
- Projects by the folks at Astral, specifically UV and RF, which are building Rust-based tooling for Python users.
- Projects by the folks at Dino (Deno) for TypeScript users.
- Temporal's core SDK stuff.
- Projects building on WASM (WebAssembly).

### Core FFI Binding Libraries

These libraries are crucial components for implementing the necessary "thin Foreign Function Interface (FFI) shims" and serve as foundational tools for cross-language development. Their strong ecosystems and documentation provide essential resources for developers.

The specific FFI libraries used by the Boundary team (BAML) and mentioned as key parts of the ecosystem are:

| Target Language | Library/Interface |
|-----------------|-------------------|
| Python | pyo3 |
| TypeScript/Node.js | N API |
| WebAssembly | wasm-bindgen/wit-bindgen |

The stability and maturity of toolchains, particularly the pyo3 ecosystem, have been noted as impressive, sometimes working "out of the box" for complex tasks like cross-compilation for multiple architectures. However, it is important to note that FFI providers occasionally ship major internal API changes (e.g., PyO3 undergoing two substantial changes and NAPI moving from version two to three).



---

## The Unified Core Model — Architectural Patterns for Building Cross-Language Libraries in Rust

### 1.0 Introduction: The Modern Imperative for Cross-Ecosystem Tooling
Contemporary software development is defined by a fragmented landscape of programming languages. Teams are routinely tasked with delivering consistent, high-performance, and strongly-typed functionality across diverse ecosystems like Python, TypeScript, and Ruby. This polyglot reality presents a significant architectural challenge: how can an organization provide a unified, high-quality developer experience without incurring the immense cost and complexity of re-implementing core logic for each target platform? The pursuit of a single source of truth that is both performant and maintainable has become a modern imperative.
This whitepaper advances the thesis that a unified architectural model, consisting of a high-performance core library written in Rust and exposed via thin Foreign Function Interface (FFI) shims, presents a powerful and viable strategy to solve this challenge. This "Unified Core Model" centralizes complex business logic, parsers, compilers, and runtimes into a memory-safe, computationally efficient foundation, while delivering idiomatic, native-feeling libraries to developers in their language of choice.
To illustrate the principles and patterns of this approach, this paper will analyze the successful implementation of this model by the development team behind BAML, a programming language for AI applications. The BAML project required the creation of a sophisticated compiler and runtime that could provide a consistent, strongly-typed developer experience across Python, TypeScript, and Ruby. Their journey provides a practical case study in navigating the trade-offs and design decisions inherent in this architecture. This analysis will move beyond the strategic justification for this model and delve into the critical patterns for execution.
### 2.0 The Strategic Rationale for a Unified Rust Core
The decision to adopt a foundational architecture for a multi-language project is a high-stakes choice that extends far beyond technical elegance. This decision is a primary determinant of team velocity, long-term maintainability, performance characteristics, and, most critically, the quality of the end-user experience. Choosing an architecture that can confidently support complex requirements while empowering a small team is a key strategic advantage.
The BAML project provides a clear example of the business and technical problem that drives the adoption of the Unified Core Model. The objective was to create a new programming language for AI, complete with a parser, compiler, diagnostic tools, and a runtime. This complex system then needed to be made accessible to developers in Python, TypeScript, and Ruby, who have high expectations for a first-class tooling experience, including features like strong typing and autocompletion. The challenge was to deliver this sophisticated, consistent experience without fracturing development efforts or compromising on quality.
#### Evaluating Architectural Alternatives
Before landing on the Rust-core model, several alternative architectures were considered, each with significant drawbacks for this specific use case:
• Native Re-implementation: This approach involves implementing the core logic multiple times—once in Python, once in TypeScript, once in Ruby, and so on. While it guarantees a perfectly idiomatic user experience, it creates an unsustainable maintenance burden, multiplies development costs, and makes it nearly impossible to ensure feature parity and consistent behavior across platforms.
• C/C++ Core: Using a C or C++ core with FFI bindings is a traditional approach to this problem. However, it introduces the significant risks of memory safety bugs and the complexities of managing legacy build systems, which can hinder development velocity and introduce a class of errors that Rust is designed to prevent.
• Subprocess/RPC Model: Exposing the core logic through Inter-Process Communication (IPC) or Remote Procedure Call (RPC) interfaces decouples the components but introduces substantial runtime overhead, serialization costs, and a more complicated deployment story for the end-user.
The Rust-based Unified Core Model was ultimately selected as the superior choice, providing the performance of a native core without the typical safety and tooling compromises.
#### The Foundational Advantages of Rust
Rust was chosen as the foundational layer for its unique combination of performance, safety, and a modern toolchain. These attributes provided the necessary confidence and capability for a small team to undertake such an ambitious project.
• Foundational Confidence: For building complex components like parsers, compilers, and asynchronous runtimes, Rust provides the low-level control and high-level abstractions necessary to implement them effectively. It serves as a "really good foundational layer" that enables teams to tackle hard problems with confidence.
• Robustness and Reliability: Rust's strict type system and ownership model eliminate entire categories of common bugs at compile time. This leads to a highly reliable core where regressions are rare. The experience of the BAML team demonstrates that this shifts the engineering focus; bugs arise not from implementation flaws, but from incorrect high-level modeling of a problem or from unexpected changes in external dependencies. This elevates the team's efforts from low-level debugging to higher-value architectural work.
• Team Empowerment: The Rust ecosystem proved to be a significant force multiplier. A small team of six engineers was able to build and maintain a large-scale system of approximately 150,000 lines of code, three of whom had no prior Rust experience just two years prior. This level of productivity and quality would have been exceptionally difficult to achieve with other technologies, demonstrating that Rust can empower small teams to deliver large-scale, high-quality systems.
While the strategic benefits are clear, this powerful architectural model is not without inherent architectural trade-offs that demand rigorous management. The next section will address these complexities and the tools available to manage them.
### 3.0 Navigating the Implementation Trade-offs and Toolchain Complexity
While the Unified Core Model is strategically sound, its implementation introduces significant complexity that must be understood and actively managed. This model, however, is not without inherent architectural trade-offs that demand rigorous management. This complexity manifests in two primary domains: the experience of the library's end-users and the internal development and release processes for the maintainers.
#### User-Facing Complexity
When a library is not implemented in the user's native language, a degree of cross-language friction is inevitable. End-users may encounter unexpected behavior or integration issues that require deeper debugging than with a pure-native library. These challenges often stem from the interaction between the Rust core and the specific runtime environments of the target languages.
• Runtime Interactions: The BAML project encountered platform-specific issues such as difficult-to-debug interactions with the Python Global Interpreter Lock (GIL).
• Build Tooling Conflicts: In the JavaScript ecosystem, the library had to contend with the complexities of the TypeScript bundler and various NodeJS loaders.
• Platform-Specific Mechanisms: Other ecosystems present unique challenges, such as integrating with Go's idiosyncratic package download and management system.
These issues require a commitment from the maintenance team to work closely with users to diagnose and resolve deep, platform-specific problems.
#### Internal Development Complexity
The most significant burden of this model falls on the internal development team, which must manage a sophisticated cross-compilation and release toolchain.
• Toolchain Overhead: The process is far more involved than standard packaging in a language like Python, which can be as simple as bundling source files into a zip archive. It requires a full cross-compile cycle, which can be difficult to set up and debug, especially when developing on one OS (e.g., macOS) and targeting another (e.g., Linux).
• Build Matrix: The need to support multiple platforms creates a combinatorial explosion of build targets. For BAML, supporting four target languages across six machine architectures results in 24 distinct binary builds for every release.
• CI/CD Pipeline: This extensive build matrix significantly complicates the Continuous Integration/Continuous Deployment (CI/CD) pipeline, requiring more sophisticated configuration and longer runtimes than a single-language project.
#### Justification for Accepting the Complexity
Despite these challenges, the trade-offs are justifiable because the modern Rust ecosystem provides powerful tools and established patterns to manage this complexity.
• The Power of cargo and build.rs: Rust's built-in build system, cargo, and its build-script functionality (build.rs) provide a "powerful and flexible foundation" for managing complex build processes. This avoids the need to rely on older, more arcane systems like CMake and provides a unified, modern interface for compilation logic.
• Maturity of FFI Ecosystems: The libraries that bridge Rust to other languages—such as pyo3 for Python, n-api for TypeScript, and Magnus for Ruby—have reached a high level of maturity. In one "unbelievable moment" during the BAML project, a complete multi-architecture cross-compile workflow for Python succeeded on the very first attempt, a testament to the robustness of the pyo3 ecosystem.
• Availability of Prior Art: Teams adopting this model are not working in a vacuum. There is a wealth of prior art from established projects at companies like Astral (Ruff, uv), Deno, Prisma, and Temporal. These projects provide proven, production-tested patterns and solutions that can be studied and adapted, significantly de-risking the development process.
With a clear understanding of these trade-offs, we can now turn to the specific architectural patterns that are crucial for successfully designing and implementing cross-language APIs with a Rust core.
### 4.0 Core Architectural Patterns for Cross-Language API Design
Success with the Unified Core Model depends on a series of critical design decisions made at the boundary between Rust and the target language. An implementation that is technically correct but ergonomically poor will fail to gain adoption. This section details three core architectural patterns related to API ergonomics, asynchronous operations, and software versioning that are essential for building libraries that feel native and intuitive to end-users.
#### 4.1 Pattern 1: Prioritize End-User Ergonomics Over Rust Idioms
A central design dilemma when creating FFI bindings is whether to expose Rust-backed objects directly to the user or to design APIs that operate on native objects from the target language (e.g., pure Python objects). While exposing Rust objects can seem more direct and performant, it often violates user expectations and breaks integration with the broader ecosystem.
##### Case Study: Error Handling
Consider a function that accepts an options object. If that object is implemented in Rust and deserialized using serde, an error for a missing field will be a standard serde error.
**Rust serde Error**
```
Expected Python pydantic Error
```
runtime error missing field repeat
A Pydantic-style validation error explaining that the required field repeat is missing from the input. It would also specify that the field crab_emoji was expected but not provided, giving the user actionable context.
To a Rust developer, the serde error is perfectly clear. To a Python developer accustomed to libraries like Pydantic, it is cryptic and unhelpful. They expect rich, contextual validation errors that pinpoint exactly what is wrong with their input data. Forcing a Rust-idiomatic error into a Python environment creates a poor user experience.
##### Case Study: Ecosystem Integration
This friction extends beyond error handling. If a user tries to use a Rust-backed options object with a standard Python feature like the multiprocessing module, they will likely encounter a runtime error such as cannot pickle builtins.rust_options object. This is because the Rust object does not automatically implement the pickle protocol that Python's ecosystem tooling expects. While it is possible to manually implement this functionality in Rust, it represents additional work required to meet baseline user expectations.
The architectural recommendation is clear: unless performance in a tight, critical loop is the absolute highest priority, design APIs to work with native language constructs. This approach ensures that the library integrates seamlessly with the target ecosystem's features "out of the box," providing a superior, more intuitive user experience that aligns with developer expectations.
#### 4.2 Pattern 2: Bridge Asynchronous Operations with Native Primitives
One of the most powerful features of Rust is its compile-time lifetime and borrow checking guarantees. However, these concepts do not exist in the runtimes of dynamic languages like Python or TypeScript. Therefore, attempting to perfectly translate Rust's lifetime semantics across the FFI boundary is often a counterproductive goal, especially for complex asynchronous operations like streaming.
##### The Challenge of Async Streaming
A naive attempt to expose a Rust stream to Python might involve wrapping the Rust stream in a Python object and calling its next method. This approach will almost certainly fail with a classic Rust compiler error: borrowed data escapes outside of method. The compiler correctly identifies that the lifetime of the stream cannot be statically coupled to the lifetime of the future being returned to the Python event loop. While patterns like Arc<Mutex> or channels are valid Rust idioms, they feel like workarounds for a problem that is fundamentally about the impedance mismatch between Rust's static lifetime analysis and the dynamic nature of the target language's event loop.
##### The Recommended Solution: Callbacks and Singletons
A more robust and idiomatic pattern is to use the native asynchronous primitives of the target language as the "glue" that bridges the two worlds.
1. Expose a Native Interface: Provide the user with a standard, familiar primitive, such as a Python async generator. This immediately meets user expectations.
2. Use Callbacks as the Bridge: The underlying Rust function should be designed to accept a callback object from the target language (e.g., a Python callable).
3. Leverage a Singleton Runtime: Within the Rust function, obtain a handle to the singleton Tokio runtime and spawn the core asynchronous logic as a new future on that runtime.
4. Pin the Stream to a Future's Stack: Create the Rust stream on the stack of the newly spawned future. This is the critical step that pins the stream's lifetime to that of the future, resolving the borrow-checking issue without requiring heap-allocated synchronization primitives like Arc<Mutex>.
5. Transfer Data: As the Rust stream yields items, acquire the Python GIL and invoke the callback, passing the data across the FFI boundary. This effectively uses the Python-side queue or generator as a channel for the Rust-side logic.
The core principle of this pattern is to let each language do what it does best. The high-performance, complex asynchronous logic remains encapsulated within a well-defined Rust future, while the user-facing API and cross-boundary data transfer are handled by the native, idiomatic primitives of the target language.
#### 4.3 Pattern 3: Simplify Release Management with Lockstep Versioning
A final, crucial decision concerns the software lifecycle: should the Python, TypeScript, and Ruby packages be versioned independently, or should all packages share a single, synchronized version number? While independent versioning offers the flexibility to ship wrapper-only patches, it introduces significant operational and communication overhead.
The recommended solution is to adopt a lockstep versioning strategy, where all packages are released with the same version number simultaneously. After over a year and a half of practice on the BAML project, this "easy way out" has proven to be a highly effective and robust decision.
The primary benefits are clarity and simplicity:

- **Clarity in Communication**: A single version number eliminates any "mental translation" when a user reports a bug. An issue reported on BAML 0.205.0 is unambiguous, regardless of whether the user is working in Python or TypeScript.
- **Simplified Triage**: The development team can immediately identify the exact state of the core codebase associated with a user's issue without needing to consult a complex mapping of wrapper versions to core versions.
For most teams building cross-language tooling, the operational simplicity and communication clarity gained from lockstep versioning far outweighs the minor inflexibility of not shipping wrapper-only patches independently. Teams must accept this trade-off, even when it "feels bad" in the short term to delay a simple wrapper fix for a full core release, because the long-term logistical gains are immense.
### 5.0 Conclusion: A Viable Architectural Model for High-Performance Tooling
The Unified Core Model, architected with a high-performance Rust core and thin FFI shims, offers a compelling and proven path to building reliable, performant, and type-safe libraries for multiple ecosystems. By centralizing complex logic in a single, robust codebase, organizations can achieve feature parity, reduce maintenance overhead, and deliver a consistent experience to a wide range of developers.
This approach is not without its challenges. It introduces undeniable complexity in the build toolchain and requires careful consideration at the API boundary to avoid creating a leaky or unintuitive abstraction. However, as this whitepaper has demonstrated, these are solvable problems. With the right architectural patterns—prioritizing end-user ergonomics over Rust idioms, using native asynchronous primitives as a bridge, and simplifying the release process with lockstep versioning—teams can successfully navigate these trade-offs. The maturity of the Rust compiler, its build tooling, and its FFI ecosystems provides the solid foundation needed for such an undertaking.
For engineering leaders and architects aiming to build the next generation of foundational developer tools, this architectural model represents an effective and strategic choice. It provides a blueprint for delivering high-quality, cross-platform software in an increasingly polyglot world, empowering small teams to build ambitious systems with confidence and precision.


---

## Rust for Everyone Else: The Secret Challenges of Building Multi-Language Tools
Modern software development is a vibrant, multi-language ecosystem. Teams want to use the best tool for the job: a high-performance, memory-safe language like Rust for core logic, while still providing libraries for developers working in Python, TypeScript, or Ruby. This "write once, run anywhere" dream is incredibly powerful, but it comes with its own set of hidden complexities.
Consider the case of BAML, a new programming language for calling AI applications. Built by a small team, they needed to create a robust compiler and runtime that could be used by developers across the AI landscape—a space dominated by Python and TypeScript. This made their multi-language strategy critical. They made a key architectural decision to build their foundation in Rust.
"the approach that we landed on of having the core logic everything from the compiler the di error diagnosis the runtime and putting all that in Rust and then using thin FFI foreign function interface shims"
This "Foreign Function Interface," or FFI, is the key. You can think of an FFI as a carefully constructed bridge or translator that allows code written in one language (like Rust) to talk to code written in another (like Python). While this sounds straightforward, building a bridge that is safe, easy to use, and reliable requires navigating some surprising design challenges.
This article explores the three most critical lessons learned by the BAML team on their journey to ship Rust to a multi-language world, providing a guide for anyone looking to build similar bridges.
The first major decision a developer must make is about the user's first impression of the tool—the API itself. This choice sets the tone for the entire user experience.
### 1. The First Big Question: How Should Your API "Feel"?
When building a library in Rust for Python users, you face a fundamental choice: should the objects your users interact with be "Rust-backed" (implemented entirely in Rust) or "native" to their language (like a standard Python class)?
The initial temptation for a Rust developer is to do as much as possible in Rust. It feels safer, more performant, and keeps everything consistent with the core logic. However, this Rust-centric view can create a confusing and frustrating experience for the end user.
Imagine a simple function, make_lorem_ipsum, that takes an options object. A Python user might want to load these options from a JSON file. If the options object is a pure Rust struct exposed to Python, things can go wrong in subtle but jarring ways. When a user makes a mistake—like providing a JSON file with a missing field—the error they receive can be completely alien.
# Confusing 'Rust' Error
```
RuntimeError: missing field `repeat`
```

# Expected 'Python' Error
```
pydantic.ValidationError: 2 validation errors for Options
repeat
  field required (type=value_error.missing)
crab_emoji
  field required (type=value_error.missing)
```
The "Rust" error comes from serde, the standard deserialization library in Rust. While it makes perfect sense to a Rust developer, it provides no context for a Python developer who has likely never heard of it. The "Python" error, by contrast, comes from pydantic, a standard data validation library in the Python ecosystem. It's verbose, clear, and formatted exactly how a Python developer would expect, making it immediately understandable.
The problems don't stop with error messages. Ecosystem integrations can also break. For example, a common pattern in Python is to "pickle" an object—serializing it to be sent to another process for parallel computation. If a user tries to pickle a generic Rust-backed object, they'll be met with another cryptic error: cannot pickle builtins rust options object. The Python tooling has no idea how to handle this foreign object type.
By using native Python objects, you allow developers to tap into a rich ecosystem of tools they already use. They get validation with pydantic and serialization with pickle "for free," because your library is handing them an object that speaks their language.
The primary lesson is to design for your users' world, not your own. While it might be slightly less performant to use native language objects that get translated at the FFI boundary, the massive gain in usability, predictability, and ecosystem compatibility is almost always worth the trade-off.
---

### 2. The Async Puzzle: Bridging Two Worlds Without a Borrow Checker
Rust's borrow checker is its safety superpower. It enforces strict rules about ownership and lifetimes at compile time, preventing a whole class of common bugs. The problem? These rules don't exist in languages like Python or TypeScript. You cannot rely on a compiler to help your users use your API correctly, which means you must design APIs that are impossible to misuse.
Consider the challenge of creating an asynchronous HTTP stream. The goal is to let a Python user iterate over chunks of data as they arrive from a server.
A naive, Rust-centric approach would be to create a Python object in Rust that wraps a Rust Stream. When the Python code asks for the next item, the Rust code would try to pull from the stream and send the data back. This immediately runs into one of the most classic async Rust errors: "borrow data escapes outside a method". The Rust compiler correctly points out that it cannot guarantee the Stream will live long enough, because its lifetime is not tied to the Python code that is calling it.
Instead of fighting the borrow checker, a more robust and user-centric solution embraces the patterns of the target language.
1. Embrace Native Patterns: Don't invent a new, foreign stream object. Instead, provide the user with a standard Python async generator, a construct they already know how to use.
2. Use Callbacks as a Bridge: The core Rust logic takes a Python callback function as an argument. It then spawns the stream processing in its own managed async task (on a single, shared Rust async runtime, which the speaker refers to as a "singleton Tokyo runtime").
3. Send, Don't Share: As the Rust task receives data from the stream, it simply calls the Python callback to send the data across the FFI bridge. This completely avoids the complex lifetime and borrowing problems, as no object's lifetime needs to span the boundary. Rust manages its stream, and Python receives data through a familiar mechanism.
Don't try to reimplement the target language's core features (like async iterators or event loops) in Rust. Instead, use the FFI layer as a simple channel to send data to native constructs that the user already knows and trusts. Let Rust handle the high-performance logic, and let Python handle the Pythonic user interface.
---

### 3. The Hidden Task: Keeping Your Sanity with Versioning
When you have one core Rust library that powers packages in four different languages, a seemingly simple question becomes a major headache: how do you manage version numbers?
There are two main strategies, each with its own trade-offs:
• Independent Versioning: The Python package could be v1.2.3, while the TypeScript package is v1.2.5. This seems flexible, as it allows you to ship a hotfix for one language without having to release a new version for all the others.
• Lockstep (Synchronous) Versioning: When you release, all packages get the exact same version number. A change to the TypeScript wrapper means v1.3.0 is released for Python, TypeScript, and Ruby simultaneously.
After spending a while talking it over, the BAML team chose what they called "the easy way out": lockstep versioning. A year and a half later, this decision proved to be surprisingly powerful. The reason had nothing to do with code and everything to do with communication.
"we've never gotten confused when we're mapping the issue the version a user is on into our internal version...it's easy to communicate when you don't have to do any kind of translation of numbers"
When a user reports a bug on version 0.205.0, the team instantly knows which version of the core code to look at. There is no need to ask, "Wait, is that the Python package version or the TypeScript version? What core version does that map to?" This eliminates a whole category of potential confusion.
The massive reduction in cognitive overhead and communication friction—for both users and the internal team—far outweighed the perceived benefit of versioning flexibility. The lesson is that for a small team, reducing cognitive load is a critical optimization. The "technically pure" solution of independent versioning would have created a tax of constant mental translation, whereas lockstep versioning created clarity.
--------------------------------------------------------------------------------
---

## The Art of Building Bridges

The journey of building multi-language tools with a Rust core is not just about technical implementation; it's an exercise in thoughtful design and empathy for the end user. The BAML team's experience reveals three core principles for success:
• Design for Their World, Not Yours: Always prioritize the user's ergonomic expectations and native language patterns over what feels idiomatic in Rust. A familiar error message is better than a slightly faster object.
• Embrace the Glue: Use the native features of the target language as the "glue" for complex interactions like async. Let the FFI be a simple data pipe, not a complex re-implementation of another language's runtime.
• Optimize for Humans: Simplify processes like versioning to reduce confusion and make communication effortless. The easiest path is often the most sustainable one for the team.
Ultimately, building successful multi-language tools is the art of building robust, well-designed bridges between ecosystems, always using the end user's experience as the ultimate guiding principle.

---

## Shipping Rust
The process of shipping Rust code to users of other languages (like Python, TypeScript, and Ruby) involves several critical design decisions, each carrying specific tradeoffs related to complexity, user experience, performance, and internal toolchain management.
The key design decisions and their associated tradeoffs, as outlined in the sources, are detailed below:
### 1. Core Implementation Strategy

When building foundational software for users in other languages, an initial decision involves how the core logic will be implemented and exposed.

| Design Decision | Description | Tradeoffs/Considerations |
|-----------------|-------------|--------------------------|
Optin a rejected

| Option B: Core Engine Bridge | Implement the core logic in a central language (like C/C++ or Rust) and expose it via Foreign Function Interface (FFI) bindings. Alternatively, use a subprocess approach exposing functionality via Inter-Process Communication (IPC) or Remote Procedure Calls (RPC). The decision to put the core logic in Rust and use thin FFI shims provides a really good foundational layer, offering confidence, strong async capabilities, and effective error handling. | Tradeoff: This choice introduces complexity for both internal developers and users. Internally, it requires significant effort on the tool chain and managing complicated Continuous Integration (CI) processes, such as performing 24 different builds to cover four targets (Python, TypeScript, Ruby, Go) across six different architectures. |
### 2. API Object Implementation

A major decision is whether to define objects in the Rust core and expose them to the target language, or to leverage native objects in the target language and pass them to Rust. This affects integration, learning curve, and performance.

| Design Decision | Consideration | Tradeoffs/Implications |
|-----------------|---------------|-------------------------|
| Rust-backed Objects | Implement target language objects (e.g., Python objects) directly in Rust (using tools like pyo3). | Tradeoff: Ecosystem Integration Errors. When things go wrong, users receive Rust-centric errors (like Serde JSON errors: "missing field repeat") that are normal in "Rustland" but highly confusing for developers accustomed to native errors (like Pydantic validation errors in Python). Tradeoff: Feature Compatibility. Core expected functionality, such as serialization (e.g., Python's pickle), may fail, resulting in errors like "cannot pickle builtins rust options object". Implementing the necessary expected functionality in Rust requires additional work for the maintainers. |
| Native (Python-backed) Objects | Define the required objects as native structs or classes in the target language (e.g., a class defined in Python) and pass them into the Rust core. | Tradeoff: Performance. This approach is slightly more expensive performance-wise because it requires an extra step of translation handled by glue code. Benefit: User Ergonomics. This design prioritizes user ergonomics, leveraging native language constructs and providing users with the error messages and functionality they expect out of the box. This approach is preferable if performance is not a super critical requirement for a given method. |
### 3. Handling Lifetimes and Asynchronous Operations

The absence of a borrow checker in languages like Python and TypeScript fundamentally changes how you must design APIs that involve state or asynchronous streaming.

| Design Decision | Consideration | Tradeoffs/Implications |
|-----------------|---------------|-------------------------|
| Encoding Rust Lifetimes | Attempting to implement core target language interfaces (like the Python async iterator protocol) in Rust by wrapping a Rust stream object. | Tradeoff: Complexity and Non-Idiomatic Rust. This leads to common async errors like "borrow data escapes outside a method" because the stream's lifetime is not statically coupled to the future being sent back. Solving this requires using non-idiomatic Rust patterns (e.g., using Arc<Mutex> or copying data). |
| Using Native Constructs (Callbacks/Singletons) | Use a more native construct (e.g., a Python async generator) and employ callback-driven streams and singletons as "magic glue" to transfer data across the FFI layer. The stream object can be pinned to the stack of the spawned future, avoiding the need for complex internal synchronization or data copying. | Tradeoff: Mixed Language Codebase. This requires writing some Python (or target language) code to solve the problem, rather than keeping the entire solution purely in Rust. Benefit: User Experience. Since Python users do not rely on or care about Rust lifetimes, this approach avoids difficult lifetime management issues at the FFI boundary and ensures the API is easier for them to use. |
### 4. Versioning Scheme

When releasing a core library wrapped for multiple ecosystems (Python, TypeScript, Ruby, Go), deciding how to handle version numbers is crucial for maintenance and user communication.

| Design Decision | Consideration | Tradeoffs/Implications |
|-----------------|---------------|-------------------------|
| Independent Versioning | Version each target language wrapper independently so that a Python-specific fix only bumps the Python package version. | Tradeoff: Communication Confusion. This complicates maintenance and debugging, as developers have to mentally translate which version of the core corresponds to a version reported by a user in a specific wrapper (e.g., Python 1.2.3 vs. Ruby 4.5.6). |
| Synchronous (Lockstep) Versioning | Ship all distributions in lockstep, bumping all versions simultaneously whenever a change is made to any part of the library or a wrapper. | Tradeoff: It "feels a little bad" when making a minor wrapper-only update, as it forces the core version to update unnecessarily. Benefit: Simplification and Clarity. This approach simplifies communication when users report issues, eliminating the need for translation or confusion regarding the internal core version being used. The benefit in simplifying user communication was deemed a "big win" and worth the trade-off. |---

## The Empathy Bridge: Designing Rust APIs for a Non-Rust World
Why would a developer write a library in Rust for users of other languages like Python or TypeScript? For many, the answer is that Rust provides an incredibly powerful and stable foundation for building complex, high-performance software.
Consider the case of BAML, a company building a new programming language for interacting with AI. To create the necessary parser, compiler, and runtime, they needed a "really good foundational layer." They chose Rust. This choice gave their small team of six people—three of whom were new to the language—the ability to build a massive, high-quality project of 150,000 lines of code. The key benefits they gained were confidence in their code, effective async and error handling, and exceptional stability with rare regressions. For them, Rust wasn't just a good choice; it was a force multiplier. In their words, "the extent and scope of what we've built would not have been possible certainly not with the level of quality that we've done without Rust."
While Rust proved to be a powerful core, making its capabilities accessible and ergonomic for developers in other ecosystems is a significant challenge. It requires careful and empathetic design choices that bridge the gap between Rust's world and the worlds of Python, TypeScript, and others.
1. The Core Dilemma: "Guest" Objects vs. "Native" Objects
When designing a cross-language API, a developer faces a central design decision: Should the API expose Rust-implemented objects directly to the user, or should it work with objects that are native to the user's language (e.g., standard Python classes)?
This isn't just an implementation detail; it's a fundamental choice that impacts how hard your API is to learn, how well your types integrate with the rest of the ecosystem, and the performance implications of crossing the language boundary. This is a choice between what feels natural for the Rust developer and what provides the best user experience (ergonomics) for the Python, TypeScript, or Ruby developer on the other side. The guiding principle should always be the user's perspective.
"You want to design for your users and you really really want to prioritize the ergonomics for your users."
| Approach | Key Characteristics for the End-User |
|----------|-------------------------------------|
| Rust-backed Objects | Can lead to unfamiliar error messages and poor integration with the existing ecosystem (e.g., standard libraries). May offer higher performance in some cases. |
| Language-native Objects | Provides expected error messages and seamless integration with ecosystem tools. Prioritizes user ergonomics over raw performance. |
The best way to understand this trade-off is to look at what happens when things inevitably go wrong.
2. Case Study 1: The User Experience of an Error Message
One of the most significant impacts of this design choice is on the error messages your users will see. Let's imagine a common scenario: a Python user tries to load an Options object from an incomplete JSON string that is missing required fields.
First, consider the error generated when using a Rust-backed object parsed with a Rust library like serde.
# Error from a Rust-backed Object
RuntimeError: missing field `repeat`
While this error is perfectly clear to a Rust developer familiar with serde, it is confusing and unhelpful for a Python developer. It only reports the first missing field it finds and provides no other context.
Now, consider the error a Python user would expect to see, which would be generated from a native Python object using a common library like pydantic.
# The Expected Python Error
pyantic.ValidationError: 2 validation errors for Options
repeat
  field required (type=value_error.missing)
crab_emoji
  field required (type=value_error.missing)
The native Python error is far superior for the user. It's familiar, and it provides crucial context about the error, such as identifying all missing fields, which is far more helpful for debugging than the Rust error that stops at the first failure. This simple comparison demonstrates how choosing to work with native objects leads to a vastly better debugging experience. But poor error messages are just one part of the problem; true integration goes deeper.
3. Case Study 2: Fitting In with the Broader Ecosystem
Developers expect libraries to work seamlessly with the standard tools and features of their chosen language. When you expose a Rust-backed object, it can feel like an alien in the user's native environment.
A perfect example of this is pickling in Python. Pickling is a standard way to serialize Python objects, often used to send them to another process for parallel computation. Imagine a Python user discovers that a function from your library is surprisingly compute-intensive. Their natural thought is, "Okay, I want to run this in a separate process." They would reasonably try to "pickle" the Rust-backed Options object to pass it as an argument. The result is an immediate failure.
### Error when pickling a Rust-backed Object
```
cannot pickle 'builtins.rust_options' object
```
The "so what?" of this error is profound. The Rust-backed object does not automatically support the protocols and interfaces that standard Python libraries expect. While it is possible for the library maintainer to go back into the Rust code and manually implement the functionality needed to make the object "pickleable," this is significant extra work. By choosing to work with native Python objects from the very beginning, this entire class of integration problems is avoided.
4. Advanced Topic: When Rust's Superpowers Don't Translate
Rust's lifetime system and borrow checker are superpowers that guarantee memory safety at compile time for Rust developers. But what happens when the users of your library are in a language like Python or TypeScript that has no borrow checker?
The core insight is this: in Rust, lifetimes are a feature for the compiler and a signal that users of your library can rely on. For a Python or TypeScript user, however, this compile-time guarantee is invisible. The lifetime transforms from a feature into a liability—an implementation burden for you, the maintainer, with zero benefit for the end-user.
The challenge of async streaming illustrates this perfectly.
1. **Naive Rust Approach**: A developer might first try to wrap a Rust stream object inside a Python object. When the Python code tries to get the next item from the stream, the Rust code attempts to poll the stream and send the result back across the FFI (Foreign Function Interface) boundary. This immediately fails with a classic Rust async error: `borrow data escapes outside a method`. The reason is that the stream's lifetime cannot be statically tied to the future being sent back to Python.
2. **User-Centric Solution**: Instead of trying to re-implement Python's internal async interfaces in Rust, the better solution is to leverage Python's native constructs.
    - First, expose a native Python async generator to the user and use a callback function to pass data from Rust into a Python queue.
    - The key is to spawn a future on a singleton Tokio runtime in your Rust code. This allows the Rust stream to be created and pinned to the stack of that future, which neatly solves the lifetime problem without resorting to complex workarounds like `Arc<Mutex>`.
    - As the Rust stream produces data, the callback is used to send that data into the Python queue. Conceptually, you are elegantly "trading the Python Q as our Rust channel," using a native Python primitive to bridge the FFI boundary.
The main lesson from this advanced example is crucial: for your non-Rust users, you don't need to encode Rust-specific concepts like lifetimes into your API. It is often far better to use the patterns and primitives native to the target language to bridge the FFI boundary.
5. A Final Piece of Wisdom: Synchronized Versioning
Designing the API is only part of the battle. You also have to manage and ship the software. When you have a single Rust core that targets multiple languages—like Python, TypeScript, Ruby, and Go—a practical question arises: how should you version the packages?
There are two main options:
1. Independent Versioning: Each language package (e.g., the Python package vs. the TypeScript package) gets its own version number.
2. Lockstep (Synchronized) Versioning: All packages share a single, unified version number. Every time a new version is released, all packages are bumped simultaneously.
The BAML team chose lockstep versioning, and after over a year, they are "really, really happy with this decision." The benefits are centered around eliminating complexity for everyone involved.
- **Clarity**: It completely eliminates confusion when communicating with users.
- **Simplicity**: When a user reports a bug on version 0.205.0, the team knows exactly what code that refers to, regardless of whether the user is on Python or TypeScript.
- **Reduced Mental Overhead**: No one needs to perform a "mental translation" from a language-specific package version to an internal core library version. A developer can copy a version number from a user's GitHub issue directly into a Slack message for the team without any ambiguity.
6. Conclusion: The Principle of User Empathy
Building a cross-language library with a Rust core is not just a technical challenge; it is fundamentally an exercise in empathy for the end-user. The goal is to leverage Rust's power without exposing its complexity to developers who live in different ecosystems. For any student of software design, this journey offers three powerful takeaways.
1. Prioritize User Ergonomics: Always consider how your API will feel to a developer in their native ecosystem. Prefer native language constructs for data objects and control flow (like async) unless performance is a critical, insurmountable blocker.
2. Design for Native Tooling: Ensure your library provides helpful, familiar error messages and integrates seamlessly with standard language features like serialization (pickle). A user should not have to know your library is written in Rust to use it effectively.
3. Simplify Everything Around the Code: Make practical choices, like synchronized versioning, that reduce complexity for both your users and your own development team during debugging and support.

