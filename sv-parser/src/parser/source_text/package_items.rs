use crate::ast::*;
use crate::parser::*;
use nom::branch::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Node)]
pub enum PackageItem {
    PackageOrGenerateItemDeclaration(Box<PackageOrGenerateItemDeclaration>),
    AnonymousProgram(Box<AnonymousProgram>),
    PackageExportDeclaration(Box<PackageExportDeclaration>),
    TimeunitsDeclaration(Box<TimeunitsDeclaration>),
}

#[derive(Clone, Debug, Node)]
pub enum PackageOrGenerateItemDeclaration {
    NetDeclaration(Box<NetDeclaration>),
    DataDeclaration(Box<DataDeclaration>),
    TaskDeclaration(Box<TaskDeclaration>),
    FunctionDeclaration(Box<FunctionDeclaration>),
    CheckerDeclaration(Box<CheckerDeclaration>),
    DpiImportExport(Box<DpiImportExport>),
    ExternConstraintDeclaration(Box<ExternConstraintDeclaration>),
    ClassDeclaration(Box<ClassDeclaration>),
    ClassConstructorDeclaration(Box<ClassConstructorDeclaration>),
    LocalParameterDeclaration(Box<(LocalParameterDeclaration, Symbol)>),
    ParameterDeclaration(Box<(ParameterDeclaration, Symbol)>),
    CovergroupDeclaration(Box<CovergroupDeclaration>),
    AssertionItemDeclaration(Box<AssertionItemDeclaration>),
    Empty(Box<Symbol>),
}

#[derive(Clone, Debug, Node)]
pub struct AnonymousProgram {
    pub nodes: (Keyword, Symbol, Vec<AnonymousProgramItem>, Keyword),
}

#[derive(Clone, Debug, Node)]
pub enum AnonymousProgramItem {
    TaskDeclaration(Box<TaskDeclaration>),
    FunctionDeclaration(Box<FunctionDeclaration>),
    ClassDeclaration(Box<ClassDeclaration>),
    CovergroupDeclaration(Box<CovergroupDeclaration>),
    ClassConstructorDeclaration(Box<ClassConstructorDeclaration>),
    Empty(Box<Symbol>),
}

// -----------------------------------------------------------------------------

#[parser]
pub(crate) fn package_item(s: Span) -> IResult<Span, PackageItem> {
    alt((
        map(package_or_generate_item_declaration, |x| {
            PackageItem::PackageOrGenerateItemDeclaration(Box::new(x))
        }),
        map(anonymous_program, |x| {
            PackageItem::AnonymousProgram(Box::new(x))
        }),
        map(package_export_declaration, |x| {
            PackageItem::PackageExportDeclaration(Box::new(x))
        }),
        map(timeunits_declaration, |x| {
            PackageItem::TimeunitsDeclaration(Box::new(x))
        }),
    ))(s)
}

#[parser]
pub(crate) fn package_or_generate_item_declaration(
    s: Span,
) -> IResult<Span, PackageOrGenerateItemDeclaration> {
    alt((
        map(net_declaration, |x| {
            PackageOrGenerateItemDeclaration::NetDeclaration(Box::new(x))
        }),
        map(data_declaration, |x| {
            PackageOrGenerateItemDeclaration::DataDeclaration(Box::new(x))
        }),
        map(task_declaration, |x| {
            PackageOrGenerateItemDeclaration::TaskDeclaration(Box::new(x))
        }),
        map(function_declaration, |x| {
            PackageOrGenerateItemDeclaration::FunctionDeclaration(Box::new(x))
        }),
        map(checker_declaration, |x| {
            PackageOrGenerateItemDeclaration::CheckerDeclaration(Box::new(x))
        }),
        map(dpi_import_export, |x| {
            PackageOrGenerateItemDeclaration::DpiImportExport(Box::new(x))
        }),
        map(extern_constraint_declaration, |x| {
            PackageOrGenerateItemDeclaration::ExternConstraintDeclaration(Box::new(x))
        }),
        map(class_declaration, |x| {
            PackageOrGenerateItemDeclaration::ClassDeclaration(Box::new(x))
        }),
        map(class_constructor_declaration, |x| {
            PackageOrGenerateItemDeclaration::ClassConstructorDeclaration(Box::new(x))
        }),
        map(pair(local_parameter_declaration, symbol(";")), |x| {
            PackageOrGenerateItemDeclaration::LocalParameterDeclaration(Box::new(x))
        }),
        map(pair(parameter_declaration, symbol(";")), |x| {
            PackageOrGenerateItemDeclaration::ParameterDeclaration(Box::new(x))
        }),
        map(covergroup_declaration, |x| {
            PackageOrGenerateItemDeclaration::CovergroupDeclaration(Box::new(x))
        }),
        map(assertion_item_declaration, |x| {
            PackageOrGenerateItemDeclaration::AssertionItemDeclaration(Box::new(x))
        }),
        map(symbol(";"), |x| {
            PackageOrGenerateItemDeclaration::Empty(Box::new(x))
        }),
    ))(s)
}

#[parser]
pub(crate) fn anonymous_program(s: Span) -> IResult<Span, AnonymousProgram> {
    let (s, a) = keyword("program")(s)?;
    let (s, b) = symbol(";")(s)?;
    let (s, c) = many0(anonymous_program_item)(s)?;
    let (s, d) = keyword("endprogram")(s)?;
    Ok((
        s,
        AnonymousProgram {
            nodes: (a, b, c, d),
        },
    ))
}

#[parser]
pub(crate) fn anonymous_program_item(s: Span) -> IResult<Span, AnonymousProgramItem> {
    alt((
        map(task_declaration, |x| {
            AnonymousProgramItem::TaskDeclaration(Box::new(x))
        }),
        map(function_declaration, |x| {
            AnonymousProgramItem::FunctionDeclaration(Box::new(x))
        }),
        map(class_declaration, |x| {
            AnonymousProgramItem::ClassDeclaration(Box::new(x))
        }),
        map(covergroup_declaration, |x| {
            AnonymousProgramItem::CovergroupDeclaration(Box::new(x))
        }),
        map(class_constructor_declaration, |x| {
            AnonymousProgramItem::ClassConstructorDeclaration(Box::new(x))
        }),
        map(symbol(";"), |x| AnonymousProgramItem::Empty(Box::new(x))),
    ))(s)
}