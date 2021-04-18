grammar JsonPath;

path: ROOT? chain? EOF;
chain: (descent | wildcard | current | key | index)+ ;
descent: DOT DOT (keyPlain | keyBr);
wildcard: DOT? BL W BR | DOT W ;
current: CUR chain?;
index: BL (single | several | slice | filter) BR;
key: DOT? keyBr | DOT keyPlain;

keyPlain: (ALAPHABET | NUM | PL |PR  | Q | COL | C | LOGIC | H | CL | CR )+ ;
keyBr :  BL STRING_QT BR  ;

single: NUM;
several:  STRING_QT (C STRING_QT)+ | NUM (C NUM)+ ;
slice:  NUM? COL NUM? COL? NUM? ;
filter:  Q PL op sign op PR ;

op: chain | STRING_QT ;
sign: ALAPHABET | LOGIC;

H        : '#';
CL       : '{';
CR       : '}';
PL       : '(';
PR       : ')';
BL       : '[';
BR       : ']';
DOT      : '.';
CUR      : '@';
ROOT     : '$';
Q        : '?';
QT       : '\'';
COL      : ':';
W        : '*';
C        : ',';
LOGIC    : '>' | '<' | '>=' | '<=' | '==' | '!=' | '~='  ;
NUM      : '-'? INT (DOT [0-9] +)? EXP? ;
STRING_QT: QT (ESC | SAFECODEPOINT)* QT;
ALAPHABET: [a-zA-Z_|]+;
WS       : [ \t\r\n\u000C]+ -> skip;


fragment ESC: '\\' (['"\\/bfnrt] | 'u' HEX HEX HEX HEX) ;
fragment HEX: [0-9a-fA-F];
fragment SAFECODEPOINT: ~ ['"\\\u0000-\u001F];
fragment INT : [0-9] [0-9]*;
fragment EXP : [Ee] [+\-]? INT;

