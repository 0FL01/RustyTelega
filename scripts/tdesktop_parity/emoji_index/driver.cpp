// Prints Telegram Desktop's emoji list in sprite order: the index, the
// code points, and the postfixed / variated / colored flags. Built from
// tdesktop's codegen (codegen/emoji/data*.cpp) by build_set.sh.
#include "codegen/emoji/data.h"
#include <QCoreApplication>
#include <cstdio>
int main(int argc, char *argv[]) {
	QCoreApplication app(argc, argv);
	auto data = codegen::emoji::PrepareData(QString::fromUtf8(argv[1]), {});
	int i = 0;
	for (const auto &e : data.list) {
		QString hex;
		for (auto c : e.id.toUcs4()) {
			if (!hex.isEmpty()) hex += '-';
			hex += QString::number(c, 16);
		}
		printf("%d\t%s\t%d\t%d\t%d\n", i++, hex.toUtf8().constData(), e.postfixed, e.variated, e.colored);
	}
	return 0;
}
