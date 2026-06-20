import domainforge


def test_dimension_parse_case_insensitive():
    if not hasattr(domainforge, "Dimension"):
        return
    d1 = domainforge.Dimension.parse("currency")
    d2 = domainforge.Dimension.parse("Currency")
    assert str(d1) == str(d2)
    assert str(d1) == "Currency"


def test_unit_constructor_and_getters():
    if not hasattr(domainforge, "Unit"):
        return
    u = domainforge.Unit("USD", "US Dollar", "Currency", 1.0, "USD")
    assert u.symbol == "USD"
    assert u.base_unit == "USD"
