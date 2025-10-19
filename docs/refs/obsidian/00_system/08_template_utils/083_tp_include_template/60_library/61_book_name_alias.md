<%*
//-------------------------------------------------------------------
// SET BOOK
//-------------------------------------------------------------------
// Books directory
const directory = "60_library/61_books/";

// Filter array to only include books in the Book Directory
const books_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: directory,
    file_class: "lib",
    type: "book",
  });

const books_obj = await tp.system.suggester(
  (item) => item.key,
  books_obj_arr,
  false,
  "Book?"
);

const book_value = books_obj.value;
const book_name = books_obj.key;

tR += book_value;
tR += ";";
tR += book_name;
%>
