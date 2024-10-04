# TODO

## Performance

- Cache created filters.

## Fixes

- Correct the implementation of <code>Display</code> wherever <code>Byte</code> is cast to <code>char</code>. Currently, it does not preserve all non-printable bytes. Use <code>::std::io::Write</code> instead.
- In <code>get_trailer</code>, build the trailer from all increments if the standard requires.
- Correct <code>XRef::parse</code> to cover hybrid-reference cases.

## Features

- Parse free and compressed objects.
- Log comments in their context.
- Implement the remaining filters/encoders.
- Process dictionaries and streams based on their <code>/Type</code> and <code>/SubType</code> so that <code>XRefStream</code> implementation of <code>Process</code> becomes a special case for <code>/Type /XRef</code>.
- Implement object streams.
- Implement features specific to linearised PDFs.
- Report object changes, like being freed, overwritten, or reused in incremental updates.
- Allow the user to specify the content of the PDF summary.
- Ensure a tolerant parser and a more restrictive validator. For example, the validator should flag all HACKs allowed in the parser as errors.
- Parse streams with data stored in an external file.
- The validator should take into account the version for each incremental update.

## Documentation

- Go through the standard again and document the code accordingly, paying attention to include the supported versions for each feature.

## Tests

- When extracting test cases containing <code>Stream</code> data from PDF files, be careful to preserve the file format (<code>dos</code> /<code>unix</code>) and file encoding (<code>utf-8</code>/<code>utf-16</code>/<code>latin1</code>/...) as changing either of these can change the stream's data.
- Include a submodule of PDF released to the public domain and use it for testing.
- Double-check test coverage of all types to cover edge cases.
- Replace panics and unwraps in tests with <code>assert_eq</code>.
- Remove redundant tests.

## Refactor

- Use <code>num_traits</code> to refactor the <code>num</code> module
- Replace <code>println!</code> and <code>eprintln!</code> with <code>log</code> calls.
- Replace <code>flate2</code> with a library that allows restricting the output size.
- For comparing files, it might be better to implement a <code>PartialEq</code> trait for <code>Stream</code> using its decoded data.
- Consider viewing <code>Escape</code> for <code>Name</code> and <code>LiteralString</code> as a <code>Filter</code>, as it is the case for <code>Hexadecimal</code>.

## Future Work

- Compare PDF files:
    - Index PDF objects for comparison.
    - Implement a <code>cmp</code> feature that allows the package to connect to a database (generate one if needed) to store and query object hashes from different files.

- Implement a PDF viewer:
    - Minimal, in a way similar to <code>Zathura</code>.
    - Yet, it provides better support for annotations, hyperlinks, and text/image selection and extraction.
