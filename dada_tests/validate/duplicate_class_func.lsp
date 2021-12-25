[
    Diagnostic {
        range: Range {
            start: Position {
                line: 2,
                character: 4,
            },
            end: Position {
                line: 2,
                character: 8,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "already have a class named `test`",
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
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_class_func.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 2,
                                character: 4,
                            },
                            end: Position {
                                line: 2,
                                character: 8,
                            },
                        },
                    },
                    message: "ignoring this function for now",
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
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_class_func.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 1,
                                character: 7,
                            },
                            end: Position {
                                line: 1,
                                character: 11,
                            },
                        },
                    },
                    message: "the class is here",
                },
            ],
        ),
        tags: None,
    },
]