import sqlite3
import xml.etree.ElementTree as ET

KANJIDIC_PATH = "src-tauri/data/kanjidic2.xml"
DB_PATH = "src-tauri/data/kanji.db"

def create_table(cursor):
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS kanji (
        character TEXT PRIMARY KEY,
        stroke_count INTEGER,
        grade INTEGER,
        jlpt_level INTEGER,
        frequency INTEGER,
        onyomi TEXT,
        kunyomi TEXT,
        meanings TEXT,
        nanori TEXT
    )
    """)

def parse_kanji(character_element):
    literal = character_element.findtext("literal", "")

    stroke_count = character_element.findtext("misc/stroke_count", "")
    grade = character_element.findtext("misc/grade", "")
    jlpt = character_element.findtext("misc/jlpt", "")
    freq = character_element.findtext("misc/freq", "")

    readings = character_element.find("reading_meaning")
    onyomi = []
    kunyomi = []
    meanings = []
    nanori = []

    if readings is not None:
        for rmgroup in readings.findall("rmgroup"):
            for reading in rmgroup.findall("reading"):
                r_type = reading.get("r_type")
                if r_type == "ja_on":
                    onyomi.append(reading.text)
                elif r_type == "ja_kun":
                    kunyomi.append(reading.text)
            for meaning in rmgroup.findall("meaning"):
                if meaning.get("m_lang") in (None, "en"):
                    meanings.append(meaning.text)

        for name in readings.findall("nanori"):
            if name.text:
                nanori.append(name.text)

    return {
        "character": literal,
        "stroke_count": int(stroke_count or 0),
        "grade": int(grade or 0),
        "jlpt": int(jlpt or 0),
        "freq": int(freq or 0),
        "onyomi": ", ".join(onyomi),
        "kunyomi": ", ".join(kunyomi),
        "meanings": "; ".join(meanings),
        "nanori": ", ".join(nanori)
    }

def insert_kanji(conn, kanji_data):
    cursor = conn.cursor()
    cursor.execute("""
        INSERT OR REPLACE INTO kanji (
            character, stroke_count, grade, jlpt_level, frequency,
            onyomi, kunyomi, meanings, nanori
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (
        kanji_data["character"],
        kanji_data["stroke_count"],
        kanji_data["grade"],
        kanji_data["jlpt"],
        kanji_data["freq"],
        kanji_data["onyomi"],
        kanji_data["kunyomi"],
        kanji_data["meanings"],
        kanji_data["nanori"]
    ))
    conn.commit()

def main():
    print("Parsing kanjidic2.xml...")
    tree = ET.parse(KANJIDIC_PATH)
    root = tree.getroot()

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    create_table(cursor)

    total = 0
    for character in root.findall("character"):
        kanji_data = parse_kanji(character)
        insert_kanji(conn, kanji_data)
        total += 1

        if total % 500 == 0:
            print(f"Inserted {total} kanji...")

    conn.close()
    print(f"Finished inserting {total} kanji!")

if __name__ == "__main__":
    main()
