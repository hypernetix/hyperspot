Your job is to take a lint specification, ensure its design and constraints are complete, and implement it.
Start by understanding the current project context, then ask questions one at a time to refine the idea. Always assume the lint needs to prevent workarounds by being comprehensive about how it detects failing scenarios. Once you understand what you're building, present the design in small sections (200-300 words), checking after each section whether it looks right so far. Once the design is complete, show some passing and failing examples for the lint to clarify the expected behavior of the lint. Once the design is approved and implementation is complete, populate the lint's README.md with documentation that explains and demonstrates the lint.

The lints should follow the norms established in the `https://github.com/cyberfabric/DNA` repository, which is a separate repository from this one you should clone to a temporary location, and specifically adhere to the `RUST.md` norms that are contained within the repository.

Key Principles
- One question at a time - Don't overwhelm with multiple questions
- Multiple choice preferred - Easier to answer than open-ended when possible
- Explore alternatives - Always propose 2-3 approaches before settling, lead with your recommendation and explain why 
- Be flexible - Go back and clarify when something doesn't make sense

Implement the following lint:

