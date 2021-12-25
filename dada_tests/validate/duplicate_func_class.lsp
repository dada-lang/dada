[
    Diagnostic {
        range: Range {
            start: Position {
                line: 2,
                character: 7,
            },
            end: Position {
                line: 2,
                character: 11,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "already have a function named `test`",
        related_information: Some(
            [
                DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url {
                            scheme: "file",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: None,
                            port: None,
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_func_class.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 2,
                                character: 7,
                            },
                            end: Position {
                                line: 2,
                                character: 11,
                            },
                        },
                    },
                    message: "ignoring this class for now",
                },
                DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url {
                            scheme: "file",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: None,
                            port: None,
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_func_class.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 1,
                                character: 4,
                            },
                            end: Position {
                                line: 1,
                                character: 8,
                            },
                        },
                    },
                    message: "the function is here",
                },
            ],
        ),
        tags: None,
    },
]