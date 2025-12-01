import sea_dsl


def test_dimension_parse_case_insensitive():
    if not hasattr(sea_dsl, "Dimension"):
        return
    d1 = sea_dsl.Dimension.parse("currency")
    d2 = sea_dsl.Dimension.parse("Currency")
    assert str(d1) == str(d2)
    assert str(d1) == "Currency"


def test_unit_constructor_and_getters():
    if not hasattr(sea_dsl, "Unit"):
        return
    u = sea_dsl.Unit("USD", "US Dollar", "Currency", 1.0, "USD")
    assert u.symbol == "USD"
    assert u.base_unit == "USD"
