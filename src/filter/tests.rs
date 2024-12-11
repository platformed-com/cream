use bumpalo::Bump;

use crate::filter::FilterRef;

use super::{
    parse_attr_path, parse_filter, parse_value_path, prelude::*, AttrPathRef, ValuePathRef,
};

const USER_EMAILS: AttrPathRef = AttrPathRef {
    urn: None,
    name: "emails",
    sub_attr: None,
};

const USER_EMAIL_VALUE: AttrPathRef = AttrPathRef {
    urn: None,
    name: "emails",
    sub_attr: Some("value"),
};

const USER_EMAIL_TYPE: AttrPathRef = AttrPathRef {
    urn: None,
    name: "emails",
    sub_attr: Some("type"),
};

const USER_NAME_FORMATTED: AttrPathRef = AttrPathRef {
    urn: None,
    name: "name",
    sub_attr: Some("formatted"),
};

const USER_EXT_EMPLOYEE_NUMBER: AttrPathRef = AttrPathRef {
    urn: Some("urn:ietf:params:scim:schemas:extension:enterprise:2.0:User"),
    name: "employeeNumber",
    sub_attr: None,
};

const USER_EXT_MANAGER_DISPLAY_NAME: AttrPathRef = AttrPathRef {
    urn: Some("urn:ietf:params:scim:schemas:extension:enterprise:2.0:User"),
    name: "manager",
    sub_attr: Some("displayName"),
};

#[test]
fn test_attr_path_parsing() {
    fn validate(input: &str, expected: AttrPathRef) {
        let path = parse_attr_path(input).unwrap();
        assert_eq!(path.as_ref(), expected);
    }

    validate("emails", USER_EMAILS);
    validate("name.formatted", USER_NAME_FORMATTED);
    validate(
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User:employeeNumber",
        USER_EXT_EMPLOYEE_NUMBER,
    );
    validate(
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User:employeeNumber",
        USER_EXT_EMPLOYEE_NUMBER,
    );
    validate(
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User:manager.displayName",
        USER_EXT_MANAGER_DISPLAY_NAME,
    );
}

#[test]
fn test_filter_parsing() {
    fn validate(input: &str, expected: FilterRef) {
        let scope = Bump::new();
        let filter = parse_filter(input).unwrap();
        assert_eq!(filter.as_ref(&scope), expected);
    }

    validate(
        "name.formatted eq \"John Smith\"",
        Compare(USER_NAME_FORMATTED, Equal, Str("John Smith")),
    );
    validate("name.formatted pr", Present(USER_NAME_FORMATTED));
    validate(
        "emails[type eq \"work\"]",
        Has(USER_EMAILS, &Compare(USER_EMAIL_TYPE, Equal, Str("work"))),
    );
    validate(
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User:manager.displayName sw \"J\" and emails[type eq \"work\"]",
        And(&[Compare(USER_EXT_MANAGER_DISPLAY_NAME, StartsWith, Str("J")), Has(USER_EMAILS, &Compare(USER_EMAIL_TYPE, Equal, Str("work")))]),
    );
}

#[test]
fn test_value_path_parsing() {
    fn validate(input: &str, expected: ValuePathRef) {
        let scope = Bump::new();
        let path = parse_value_path(input).unwrap();
        assert_eq!(path.as_ref(&scope), expected);
    }

    validate(
        "emails[type eq \"work\"]",
        Filtered(USER_EMAILS, Compare(USER_EMAIL_TYPE, Equal, Str("work"))),
    );
    validate(
        "emails[type eq \"work\"].value",
        Filtered(
            USER_EMAIL_VALUE,
            Compare(USER_EMAIL_TYPE, Equal, Str("work")),
        ),
    );
}
